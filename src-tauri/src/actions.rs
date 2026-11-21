use std::process::Command;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(windows)]
#[link(name = "user32")]
extern "system" {
    fn keybd_event(b_vk: u8, b_scan: u8, dw_flags: u32, dw_extra_info: usize);
}

#[cfg(windows)]
#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: *mut std::ffi::c_void,
        lp_operation: *const u16,
        lp_file: *const u16,
        lp_parameters: *const u16,
        lp_directory: *const u16,
        n_show_cmd: i32,
    ) -> isize;
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
fn shell_open(target: &str) -> Result<(), String> {
    let op = to_wide("open");
    let file = to_wide(target);
    let ret = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            op.as_ptr(),
            file.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            5, // SW_SHOW
        )
    };
    if ret <= 32 {
        Err(format!("Falha ao abrir '{target}' (código Win32: {ret})"))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn shell_open(_target: &str) -> Result<(), String> {
    Err("Ações locais suportadas apenas no Windows.".into())
}

fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn normalize_app_key(raw: &str) -> String {
    raw.trim()
        .to_lowercase()
        .replace('ç', "c")
        .replace(['á', 'à', 'ã', 'â'], "a")
        .replace(['é', 'ê'], "e")
        .replace('í', "i")
        .replace(['ó', 'õ', 'ô'], "o")
        .replace('ú', "u")
}

// Helper PowerShell reutilizável: espera a janela abrir de verdade (inclusive apps Store/UWP
// como WhatsApp), traz pro foco e valida por título/processo antes de enviar qualquer tecla.
const PS_WIN_HELPER: &str = r#"
Add-Type -TypeDefinition @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class AsterWin32 {
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr hWnd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern int GetWindowText(IntPtr hWnd, StringBuilder text, int count);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int X, int Y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint dwFlags, int dx, int dy, uint dwData, UIntPtr dwExtraInfo);
}
"@

function Find-AppProcess([string]$procPattern, [string]$titlePattern) {
    Get-Process -ErrorAction SilentlyContinue | Where-Object {
        $_.MainWindowHandle -ne [IntPtr]::Zero -and (
            $_.ProcessName -like $procPattern -or
            ($titlePattern -and $_.MainWindowTitle -like $titlePattern)
        )
    } | Select-Object -First 1
}

function Is-ForegroundApp([string]$procPattern, [string]$titlePattern) {
    $fg = [AsterWin32]::GetForegroundWindow()
    if ($fg -eq [IntPtr]::Zero) { return $false }
    $sb = New-Object System.Text.StringBuilder 256
    [AsterWin32]::GetWindowText($fg, $sb, 256) | Out-Null
    $title = $sb.ToString()
    if ($titlePattern -and $title -like $titlePattern) { return $true }
    $fgPid = [uint32]0
    [AsterWin32]::GetWindowThreadProcessId($fg, [ref]$fgPid) | Out-Null
    if ($fgPid -gt 0) {
        $fgProc = Get-Process -Id $fgPid -ErrorAction SilentlyContinue
        if ($null -ne $fgProc -and $fgProc.ProcessName -like $procPattern) { return $true }
    }
    return $false
}

function Focus-AppWindow($proc, $wshell, [string]$procPattern, [string]$titlePattern) {
    if (Is-ForegroundApp $procPattern $titlePattern) { return $true }
    if ($null -ne $proc) {
        $hwnd = $proc.MainWindowHandle
        if ([AsterWin32]::IsIconic($hwnd)) {
            [AsterWin32]::ShowWindow($hwnd, 9) | Out-Null
            Start-Sleep -Milliseconds 200
        }
        [AsterWin32]::SetForegroundWindow($hwnd) | Out-Null
        $null = $wshell.AppActivate($proc.Id)
        Start-Sleep -Milliseconds 200
    }
    if (-not (Is-ForegroundApp $procPattern $titlePattern) -and $titlePattern) {
        $cleanTitle = $titlePattern.Trim('*')
        $null = $wshell.AppActivate($cleanTitle)
        Start-Sleep -Milliseconds 200
    }
    return (Is-ForegroundApp $procPattern $titlePattern)
}

function Wait-AppReady([string]$procPattern, [string]$titlePattern, [int]$timeoutMs = 15000) {
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    while ($sw.ElapsedMilliseconds -lt $timeoutMs) {
        $p = Find-AppProcess $procPattern $titlePattern
        if ($null -ne $p) { return $p }
        if (Is-ForegroundApp $procPattern $titlePattern) {
            return [PSCustomObject]@{ Id = 0; MainWindowHandle = [AsterWin32]::GetForegroundWindow() }
        }
        Start-Sleep -Milliseconds 250
    }
    return $null
}
"#;

fn run_ps_hidden(script: &str, envs: &[(&str, &str)]) {
    let full_script = format!("{PS_WIN_HELPER}\n{script}");
    let envs_owned: Vec<(String, String)> = envs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

    std::thread::spawn(move || {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &full_script,
        ]);
        for (k, v) in envs_owned {
            cmd.env(k, v);
        }
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let _ = cmd.status();
    });
}

#[tauri::command]
pub fn open_app(name: String) -> Result<(), String> {
    let key = normalize_app_key(&name);

    let mapped: Option<&str> = match key.as_str() {
        "brave" | "navegador" => Some("brave"),
        "vivaldi" => Some("vivaldi"),
        "chrome" => Some("chrome"),
        "edge" => Some("msedge"),
        "discord" => Some("discord:"),
        "spotify" => Some("spotify:"),
        "whatsapp" | "zap" | "wpp" => Some("whatsapp:"),
        "vscode" | "vs code" | "code" => Some("code"),
        "steam" => Some("steam:"),
        "calculadora" | "calc" => Some("calc"),
        "bloco de notas" | "notepad" => Some("notepad"),
        "explorador" | "arquivos" | "explorer" => Some("explorer"),
        "gerenciador de tarefas" | "taskmgr" => Some("taskmgr"),
        "configuracoes" | "settings" => Some("ms-settings:"),
        "terminal" | "wt" => Some("wt"),
        _ => None,
    };

    if let Some(target) = mapped {
        return shell_open(target);
    }

    // Fallback seguro: apenas nomes simples alfanuméricos sem metacaracteres de shell
    let trimmed = name.trim();
    let is_safe = !trimmed.is_empty()
        && trimmed.len() <= 64
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '-' | '_' | '.'));

    if !is_safe {
        return Err(format!("Aplicativo bloqueado por segurança: '{trimmed}'"));
    }

    shell_open(trimmed)
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
    let trimmed = url.trim();
    if !(trimmed.starts_with("https://") || trimmed.starts_with("http://")) {
        return Err(format!("URL bloqueada (apenas http/https permitido): '{trimmed}'"));
    }
    shell_open(trimmed)
}

#[tauri::command]
pub fn media_control(action: String) -> Result<(), String> {
    #[cfg(windows)]
    {
        const KEYEVENTF_KEYUP: u32 = 0x0002;
        let (vk, repeat): (u8, usize) = match action.trim().to_lowercase().as_str() {
            "playpause" | "play" | "pause" => (0xB3, 1), // VK_MEDIA_PLAY_PAUSE
            "next" => (0xB0, 1),                         // VK_MEDIA_NEXT_TRACK
            "prev" | "previous" => (0xB1, 1),            // VK_MEDIA_PREV_TRACK
            "mute" => (0xAD, 1),                         // VK_VOLUME_MUTE
            "volup" => (0xAF, 4),                        // VK_VOLUME_UP (~+8%)
            "voldown" => (0xAE, 4),                      // VK_VOLUME_DOWN (~-8%)
            other => return Err(format!("Ação de mídia desconhecida: '{other}'")),
        };

        unsafe {
            for _ in 0..repeat {
                keybd_event(vk, 0, 0, 0);
                keybd_event(vk, 0, KEYEVENTF_KEYUP, 0);
            }
        }
        Ok(())
    }

    #[cfg(not(windows))]
    {
        let _ = action;
        Err("Controle de mídia disponível apenas no Windows.".into())
    }
}

#[tauri::command]
pub fn play_spotify(query: String) -> Result<(), String> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err("Nome da música vazio.".into());
    }

    let uri = format!("spotify:search:{}", percent_encode(trimmed));

    // Usa UI Automation filtrando pela área principal (ignora a barra lateral "Sua Biblioteca"
    // e a barra inferior) e prioriza o botão cujo nome bate com os termos da busca.
    let ps = r#"
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes

function Norm-Text([string]$s) {
    if ([string]::IsNullOrWhiteSpace($s)) { return "" }
    $n = $s.Normalize([System.Text.NormalizationForm]::FormD)
    $sb = New-Object System.Text.StringBuilder
    foreach ($ch in $n.ToCharArray()) {
        if ([System.Globalization.CharUnicodeInfo]::GetUnicodeCategory($ch) -ne [System.Globalization.UnicodeCategory]::NonSpacingMark) {
            [void]$sb.Append($ch)
        }
    }
    return $sb.ToString().ToLowerInvariant()
}

$wshell = New-Object -ComObject WScript.Shell
$wasOpen = ($null -ne (Find-AppProcess "Spotify*" "*Spotify*"))

Start-Process $env:ASTER_SPOTIFY_URI
$proc = Wait-AppReady "Spotify*" "*Spotify*" 15000
if ($null -eq $proc) { exit 0 }

if (-not $wasOpen) {
    Start-Sleep -Milliseconds 4200
    Start-Process $env:ASTER_SPOTIFY_URI
    Start-Sleep -Milliseconds 1600
} else {
    Start-Sleep -Milliseconds 1200
}

$null = Focus-AppWindow $proc $wshell "Spotify*" "*Spotify*"

$queryNorm = Norm-Text $env:ASTER_SPOTIFY_QUERY
$tokens = @($queryNorm -split '[^a-z0-9]+' | Where-Object { $_.Length -ge 2 })

$hwnd = $proc.MainWindowHandle
$btnCond = New-Object System.Windows.Automation.PropertyCondition(
    [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
    [System.Windows.Automation.ControlType]::Button
)

$playBtn = $null
for ($try = 0; $try -lt 24; $try++) {
    try {
        $root = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($null -ne $root) {
            $winRect = $root.Current.BoundingRectangle
            $minLeft = $winRect.Left + [Math]::Max(280, $winRect.Width * 0.22)
            $minTop  = $winRect.Top + 115
            $maxTop  = $winRect.Bottom - 115

            $buttons = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $btnCond)
            $bestMatch = $null
            $bestScore = 0
            $firstMainBtn = $null

            foreach ($b in $buttons) {
                $name = $b.Current.Name
                if ([string]::IsNullOrWhiteSpace($name)) { continue }
                if ($name -notmatch '^(Tocar|Reproduzir|Play)\s+.+') { continue }
                if ($name -match '(aleat.ri|shuffle|pr.xim|anterior|next|prev|playlist vazia)') { continue }

                $rect = $b.Current.BoundingRectangle
                if ($rect.Width -lt 10 -or $rect.Height -lt 10) { continue }
                # Ignora a barra lateral esquerda (Sua Biblioteca), cabeçalho e player inferior
                if ($rect.Left -lt $minLeft -or $rect.Top -lt $minTop -or $rect.Top -gt $maxTop) { continue }

                if ($null -eq $firstMainBtn) { $firstMainBtn = $b }

                $nameNorm = Norm-Text $name
                $score = 0
                foreach ($t in $tokens) {
                    if ($nameNorm.Contains($t)) { $score++ }
                }
                if ($score -gt $bestScore) {
                    $bestScore = $score
                    $bestMatch = $b
                }
            }

            if ($null -ne $bestMatch) {
                $playBtn = $bestMatch
                break
            }
            # Só usa fallback do 1º botão da área central depois de dar tempo pra nova busca renderizar
            if ($try -ge 8 -and $null -ne $firstMainBtn) {
                $playBtn = $firstMainBtn
                break
            }
        }
    } catch {}
    Start-Sleep -Milliseconds 350
}

if ($null -ne $playBtn) {
    $invoked = $false
    try {
        $pat = $playBtn.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
        $pat.Invoke()
        $invoked = $true
    } catch {}

    if (-not $invoked) {
        $null = Focus-AppWindow $proc $wshell "Spotify*" "*Spotify*"
        $rect = $playBtn.Current.BoundingRectangle
        $cx = [int]($rect.Left + $rect.Width / 2)
        $cy = [int]($rect.Top + $rect.Height / 2)
        [AsterWin32]::SetCursorPos($cx, $cy) | Out-Null
        Start-Sleep -Milliseconds 80
        [AsterWin32]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero) # LEFTDOWN
        [AsterWin32]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero) # LEFTUP
    }
}
"#;

    run_ps_hidden(
        ps,
        &[("ASTER_SPOTIFY_URI", &uri), ("ASTER_SPOTIFY_QUERY", trimmed)],
    );
    Ok(())
}

#[tauri::command]
pub fn send_whatsapp(contact: String, message: String) -> Result<(), String> {
    let contact_trim = contact.trim();
    let msg_trim = message.trim();
    if contact_trim.is_empty() || msg_trim.is_empty() {
        return Err("Contato e mensagem são obrigatórios.".into());
    }

    let digits: String = contact_trim.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.len() >= 10 {
        let phone = if digits.starts_with("55") {
            digits
        } else {
            format!("55{digits}")
        };
        let uri = format!(
            "whatsapp://send?phone={}&text={}",
            phone,
            percent_encode(msg_trim)
        );
        let ps = r#"
$wshell = New-Object -ComObject WScript.Shell
$wasOpen = ($null -ne (Find-AppProcess "WhatsApp*" "*WhatsApp*"))
Start-Process $env:ASTER_WPP_URI

$proc = Wait-AppReady "WhatsApp*" "*WhatsApp*" 15000
if ($null -eq $proc) { exit 0 }

if (-not $wasOpen) {
    Start-Sleep -Milliseconds 3500
    Start-Process $env:ASTER_WPP_URI
    Start-Sleep -Milliseconds 1200
} else {
    Start-Sleep -Milliseconds 900
}

if (Focus-AppWindow $proc $wshell "WhatsApp*" "*WhatsApp*") {
    $wshell.SendKeys('{ENTER}')
}
"#;
        run_ps_hidden(ps, &[("ASTER_WPP_URI", &uri)]);
        return Ok(());
    }

    // Espera o WhatsApp abrir de verdade (inclusive no cold start), garante foco e preserva Ctrl+C
    let ps = r#"
$oldClip = $null
try { $oldClip = Get-Clipboard -Raw -ErrorAction SilentlyContinue } catch {}
try {
    $wshell = New-Object -ComObject WScript.Shell
    $wasOpen = ($null -ne (Find-AppProcess "WhatsApp*" "*WhatsApp*"))
    Start-Process "whatsapp:"

    $proc = Wait-AppReady "WhatsApp*" "*WhatsApp*" 15000
    if ($null -eq $proc) { exit 0 }

    # Se abriu do zero, espera a lista de conversas carregar antes de digitar
    if (-not $wasOpen) {
        Start-Sleep -Milliseconds 3500
    } else {
        Start-Sleep -Milliseconds 500
    }

    # Só envia teclas se o WhatsApp estiver realmente em primeiro plano (nunca digita em outro app)
    if (-not (Focus-AppWindow $proc $wshell "WhatsApp*" "*WhatsApp*")) {
        Start-Sleep -Milliseconds 400
        if (-not (Focus-AppWindow $proc $wshell "WhatsApp*" "*WhatsApp*")) { exit 0 }
    }

    $wshell.SendKeys('^f')
    Start-Sleep -Milliseconds 450

    $wshell.SendKeys('^a{BACKSPACE}')
    Set-Clipboard -Value $env:ASTER_CONTACT
    $wshell.SendKeys('^v')
    Start-Sleep -Milliseconds 1300

    $wshell.SendKeys('{DOWN}')
    Start-Sleep -Milliseconds 250
    $wshell.SendKeys('{ENTER}')
    Start-Sleep -Milliseconds 700

    if (-not (Focus-AppWindow $proc $wshell "WhatsApp*" "*WhatsApp*")) { exit 0 }

    Set-Clipboard -Value $env:ASTER_MSG
    $wshell.SendKeys('^v')
    Start-Sleep -Milliseconds 350
    $wshell.SendKeys('{ENTER}')
} finally {
    Start-Sleep -Milliseconds 300
    if ($null -ne $oldClip) {
        try { Set-Clipboard -Value $oldClip } catch {}
    }
}
"#;
    run_ps_hidden(ps, &[("ASTER_CONTACT", contact_trim), ("ASTER_MSG", msg_trim)]);
    Ok(())
}

#[tauri::command]
pub fn send_discord_dm(user: String, message: String) -> Result<(), String> {
    let user_trim = user.trim();
    let msg_trim = message.trim();
    if user_trim.is_empty() || msg_trim.is_empty() {
        return Err("Usuário e mensagem são obrigatórios.".into());
    }

    // Espera o Discord abrir de verdade (inclusive no cold start), garante foco e preserva Ctrl+C
    let ps = r#"
$oldClip = $null
try { $oldClip = Get-Clipboard -Raw -ErrorAction SilentlyContinue } catch {}
try {
    $wshell = New-Object -ComObject WScript.Shell
    $wasOpen = ($null -ne (Find-AppProcess "Discord*" "*Discord*"))
    Start-Process "discord:"

    $proc = Wait-AppReady "Discord*" "*Discord*" 15000
    if ($null -eq $proc) { exit 0 }

    if (-not $wasOpen) {
        Start-Sleep -Milliseconds 4000
    } else {
        Start-Sleep -Milliseconds 500
    }

    if (-not (Focus-AppWindow $proc $wshell "Discord*" "*Discord*")) {
        Start-Sleep -Milliseconds 400
        if (-not (Focus-AppWindow $proc $wshell "Discord*" "*Discord*")) { exit 0 }
    }

    $wshell.SendKeys('^k')
    Start-Sleep -Milliseconds 500

    Set-Clipboard -Value $env:ASTER_USER
    $wshell.SendKeys('^v')
    Start-Sleep -Milliseconds 1100

    $wshell.SendKeys('{ENTER}')
    Start-Sleep -Milliseconds 700

    if (-not (Focus-AppWindow $proc $wshell "Discord*" "*Discord*")) { exit 0 }

    Set-Clipboard -Value $env:ASTER_MSG
    $wshell.SendKeys('^v')
    Start-Sleep -Milliseconds 350
    $wshell.SendKeys('{ENTER}')
} finally {
    Start-Sleep -Milliseconds 300
    if ($null -ne $oldClip) {
        try { Set-Clipboard -Value $oldClip } catch {}
    }
}
"#;
    run_ps_hidden(ps, &[("ASTER_USER", user_trim), ("ASTER_MSG", msg_trim)]);
    Ok(())
}
