<img width="872" height="720" alt="aster_actions" src="https://github.com/user-attachments/assets/b61fc976-78a8-49bf-b3e6-d58c374c733a" />

# Aster Companion

Assistente virtual de desktop para **Windows 11**, construída em **Tauri v2 (Rust + HTML/CSS/JS)** com foco em baixo consumo de recursos, animações interativas estilo *desktop pet* e automação nativa do sistema com IA híbrida.

> Inspirado no projeto open-source [capri-desktop](https://github.com/lachydotmcg/capri-desktop).

---

## Novidades da v1.2.0

* **Detecção real de música (Windows SMTC):** lê nativamente via WinRT o que está tocando no Spotify, YouTube ou navegador, alterna automaticamente para o sprite `music` ao dar play e volta ao `idle` ao pausar.
* **Streaming em tempo real + Auto-start do Ollama:** respostas aparecem token por token no balão e o `ollama serve` é iniciado silenciosamente em segundo plano caso esteja desligado.
* **Visão de tela sob demanda:** `/ver`, `/tela` ou *"olha minha tela"* capturam a tela principal em memória e analisam com o modelo multimodal configurado no Ollama.
* **Lembretes, timers e Pomodoro com voz:** `/pomodoro`, `/lembrete 15m ...`, *"me lembra às 22:30..."*, com alerta sonoro suave e aviso falado.
* **Dropdown automático de modelos do Ollama:** lista os modelos instalados no PC para chat e visão direto na aba **IA & Conexões**.
* **Partículas sutis opcionais + consciência de skin:** efeitos leves configuráveis na aba **Temas** e reconhecimento da skin ativa nas conversas.
* **Importador de skins (Beta):** remove fundo de sprites crus via `rembg (isnet-anime)`, aplica contorno preto opcional e extrai o ciclo de caminhada de `.mp4`.
* **Otimizações gerais:** clientes HTTP keep-alive (`OnceLock`), cache do `.env` em RAM (`RwLock`), carregamento de skins via `Blob` URLs com revogação de memória, throttle em `syncHitRects()` e pausa de loops quando oculta.

Histórico completo em [CHANGELOG.md](CHANGELOG.md).

---

## Principais Recursos

### Interface & Desktop Companion
* **Overlay transparente sem bordas** (`alwaysOnTop`) com **click-through inteligente** nas áreas transparentes ao redor da personagem.
* **System Tray (bandeja do Windows):** acesso rápido para abrir chat, configurações, passeio automático, inicialização com o Windows, sempre no topo e mostrar/esconder.
* **Modo Tela Limpa + Atalho Global `Alt + A`:** pílula de chat flutuante com ícones SVG minimalistas que se recolhe automaticamente após inatividade.
* **Animações e física:** piscar aleatório, expressões dinâmicas, caminhada em 8 frames pela barra de tarefas, física de pêndulo por inércia ao arrastar, encaixe sentado na borda de janelas abertas e interações de headpat.

### Skins & Temas
* **Sistema de Skins (`skins/`):** criação de pastas pela aba **Skins**, checklist visual (6 essenciais, 3 expressões e extras), troca ao vivo sem recompilar (`/skin natalina`, `/skin padrao`) e fallback inteligente dentro da própria skin.
* **Skin Natalina inclusa (`skins/natalina/`):** conjunto completo com 20 sprites, incluindo caminhada em 8 frames.
* **6 Presets de Cores ao vivo (aba Temas):** *Glass Gothic*, *Kinkou Emerald*, *Crimson Natal*, *Cyber Ice*, *Sakura Nocturne* e *Obsidian Mono*.

### IA Híbrida, Memória & RAG Local
* **Ollama local (texto + visão):** respostas com streaming, seleção automática de modelos instalados e suporte a visão de tela.
* **Escalonamento remoto (Hermes):** delegação de tarefas complexas via API compatível com OpenAI (`/hermes ...` ou *"pede pro Hermes..."*).
* **Memória persistente (`memory.json`):** salva, lista e apaga fatos e preferências entre sessões.
* **RAG local (`docs`):** busca em Rust nos arquivos `.txt`, `.md`, `.json`, `.csv` e `.log` colocados na pasta `docs`.
* **Comentários espontâneos:** comentários curtos ocasionais baseados no horário, janela ativa ou música tocando (pausados automaticamente em tela cheia).

### Automação no Windows 11 & Voz
* **Voz personalizada (Fish Audio TTS):** síntese de baixa latência em Rust, lip-sync real via `AnalyserNode`, controle de volume (0–100%) e botão de teste.
* **Microfone offline (STT):** captura a 16 kHz com VAD adaptativo, wake-word `"Aster"` e worker Python com `faster-whisper` (`int8` CPU).
* **Ações nativas em Rust:** abertura de aplicativos e URLs (`ShellExecuteW`, incluindo atalhos como `spot` e `dc`), controle de mídia e volume via Win32 (`keybd_event`) e leitura do clipboard (`arboard`).
* **Integrações:** Spotify via Windows UI Automation, WhatsApp, Discord DM preservando o clipboard e Discord Webhook.

---

## Stack

* **Backend & Desktop:** Rust, [Tauri v2](https://v2.tauri.app/), Win32 e WinRT (`user32`, `shell32`, `dwmapi`, `Windows.Media.Control`)
* **Frontend:** HTML5, CSS3 e JavaScript puro (WebView2 nativo do Windows 11)
* **Voz:** Fish Audio API (TTS em Rust) + Python 3.12 / `faster-whisper` (STT offline opcional)
* **IA:** [Ollama](https://ollama.com/) (local, texto e visão) + API OpenAI-compatible (Hermes)

---

## Como Rodar Localmente (Windows 11)

### Pré-requisitos
* [Node.js](https://nodejs.org/) (v18+)
* [Rust (`rustup`)](https://rustup.rs/) + Visual Studio C++ Build Tools
* [Ollama](https://ollama.com/) (`ollama pull qwen3:4b-instruct` e opcionalmente `ollama pull qwen2.5vl:3b` para visão de tela)
* *(Opcional, para microfone e importador beta de skins)* Python 3.12

### Instalação
1. Clone o repositório e instale o CLI do Tauri:
   ```powershell
   npm install
   ```
2. Copie o template de ambiente e ajuste suas configurações locais:
   ```powershell
   Copy-Item .env.example .env
   ```
3. *(Opcional)* Crie o ambiente virtual Python para o microfone offline:
   ```powershell
   python -m venv .venv
   .\.venv\Scripts\pip install -r requirements.txt
   ```
4. Rode em desenvolvimento ou gere o executável `Aster.exe`:
   ```powershell
   npm run dev        # desenvolvimento
   npm run build:exe  # compila o Aster.exe na raiz
   ```

---

## Estrutura do Projeto

```text
Aster-Companion/
├── src/                  # Frontend WebView2 (index.html, app.js, settings.html, themes.js, sprites/)
├── skins/                # Skins customizáveis ao vivo (ex: skins/natalina/)
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs        # Janela, click-through, System Tray, hotkey Alt+A, drag físico e perch
│   │   ├── actions.rs    # Win32 mídia, ShellExecuteW, Spotify UIA, WhatsApp e Discord DM
│   │   ├── brain.rs      # Ollama (streaming, auto-start, visão, lista de modelos), Hermes, webhook e clipboard
│   │   ├── config.rs     # Leitura, cache em RAM e gravação segura do .env
│   │   ├── media.rs      # Detecção nativa de música no Windows (SMTC / WinRT)
│   │   ├── memory.rs     # Memória persistente (memory.json)
│   │   ├── docs.rs       # Busca local em documentos (RAG)
│   │   ├── skins.rs      # Listagem, criação, checklist, importador beta e troca de skins ao vivo
│   │   └── voice.rs      # Worker Python (faster-whisper STT) e síntese Fish Audio (TTS)
│   └── tauri.conf.json
├── transcrever.py        # Worker offline de Speech-to-Text (faster-whisper)
├── processar_skin.py     # Importador/processador local de skins (Beta)
├── build_exe.ps1         # Script de build release -> Aster.exe
├── CHANGELOG.md          # Histórico de versões
├── .env.example          # Template de configuração (sem chaves reais)
└── requirements.txt      # Dependências Python opcionais
```

---

## Licença & Créditos

* Código-fonte inspirado em [capri-desktop](https://github.com/lachydotmcg/capri-desktop).
* **Uso pessoal / portfólio:** fique à vontade para estudar e adaptar o código; os sprites e a identidade visual da personagem **Aster** são de uso pessoal.
