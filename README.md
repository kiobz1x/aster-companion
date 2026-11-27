<div align="center">

<img width="872" height="720" alt="aster_actions" src="https://github.com/user-attachments/assets/b61fc976-78a8-49bf-b3e6-d58c374c733a" />

# Aster Companion

Assistente virtual de desktop para **Windows 11**, construída em **Tauri v2 (Rust + HTML/CSS/JS)** com foco em baixo consumo de recursos, animações interativas estilo *desktop pet* e automação nativa do sistema com IA híbrida.

<p>
  <a href="https://github.com/kiobz1x/aster-companion/releases"><img src="https://img.shields.io/badge/versão-v1.3.7-7c3aed?style=flat-square" alt="Versão v1.3.7" /></a>
  <img src="https://img.shields.io/badge/Windows-11-18181b?style=flat-square&logo=windows11&logoColor=c4b5fd" alt="Windows 11" />
  <img src="https://img.shields.io/badge/Tauri_v2-Rust-18181b?style=flat-square&logo=tauri&logoColor=c4b5fd" alt="Tauri v2 + Rust" />
  <img src="https://img.shields.io/badge/IA-Ollama_%7C_Nuvem-18181b?style=flat-square" alt="IA Híbrida" />
</p>

<p>
  <a href="https://github.com/kiobz1x/aster-companion/releases">Download (Releases)</a> ·
  <a href="COMANDOS.md">Guia de Comandos</a> ·
  <a href="CHANGELOG.md">Changelog</a>
</p>

<sub>Inspirado no projeto open-source <a href="https://github.com/lachydotmcg/capri-desktop">capri-desktop</a>.</sub>

</div>

---

## Novidades da v1.3.x (até v1.3.7)

> [!TIP]
> **Destaque da versão:** a Aster agora roda tanto **100% offline no Ollama** quanto **na nuvem (Groq, Gemini, OpenRouter ou OpenAI)**, atualiza sozinha em **1 clique** direto pelas Configurações, escuta sua voz direto com **`Alt + V`** (sem precisar instalar Python usando Groq) e abre qualquer jogo instalado na sua **Steam**.

| Novidade | O que muda na prática |
| :--- | :--- |
| **Auto-atualizador de 1 clique** | Verifica automaticamente novas versões no GitHub, avisa no balão e atualiza + reinicia sozinha pelo botão **Atualizar agora** nas Configurações — sem nunca apagar `.env`, memórias, `docs/` ou skins próprias. |
| **IA Híbrida + Listar modelos** | Escolha **Ollama local** ou **API na Nuvem** (Groq, Gemini, OpenRouter, OpenAI) com streaming SSE, visão de tela, botão **Listar modelos** (dropdown automático) e **Testar IA** (latência em `ms`). |
| **Voz direta (`Alt + V`) + Groq Whisper** | Aperte **`Alt + V`** de qualquer tela ou jogo para falar direto (sem dizer `"Aster"`). Quem usa Groq ganha microfone nativo sem instalar Python, além de síntese Fish Audio frase por frase. |
| **Modo Jogo Leve + Steam** | Acompanha partidas de **League of Legends** ao vivo, reconhece jogos populares, abre **qualquer jogo instalado na Steam** pelo nome (*"abre o Stardew Valley"*) e desativa blur/animações pesadas em jogo para preservar o FPS. |
| **Pomodoro, Fechar Apps & Personalização** | Painel visual de **Pomodoro & Timers**, fechar apps (*"fecha o Spotify"*), nome configurável (`USER_NAME`), tamanho na tela (`75%`–`130%`), balanço rítmico na música e suporte a `.webp`/`.gif`. |
| **Instalação simplificada (`Instalar_Extras.bat`)** | Pacote `.zip` organizado com pasta `Aster-Companion\` e assistente temático de 1 clique para criar atalho na Área de Trabalho e instalar opcionais sem abrir terminal. |

<details>
<summary><strong>Ver todas as melhorias técnicas da linha v1.3.x</strong></summary>

* **Sprites otimizados para `720px` (2× Retina):** redução de ~67% no tamanho dos PNGs sem perda visual, com padrão atualizado também no `processar_skin.py`.
* **Lip-sync 90% mais leve:** trocado de `requestAnimationFrame` para intervalo de `120ms`, reduzindo o uso de GPU do WebView2.
* **Voz mais natural:** pausa de corte aumentada para `1,6s`, frase máxima de `15s` e `prompt` de vocabulário no Whisper para acertar `"Aster"`.
* **Contexto e busca local aprimorados:** `num_ctx: 4096`, histórico de 18 mensagens e stemming em PT-BR no RAG local (`docs/`).
* **Configurações redesenhadas:** janela redimensionável (`760×600`) em 5 abas com links diretos para gerar chaves grátis e guia completo em [COMANDOS.md](COMANDOS.md).

</details>

---

## Download e Instalação Rápida (Windows 11 — sem programar)

> [!IMPORTANT]
> Na página de [Releases](https://github.com/kiobz1x/aster-companion/releases), baixe **APENAS** o arquivo **`Aster-Companion-v1.3.7-win64.zip`** (não precisa baixar o *Source code*).

### 1. Abrir a Aster
1. Extraia o `.zip` — ele já cria uma pasta organizada **`Aster-Companion\`** com tudo dentro.
2. Abra o **`Aster.exe`** *(ou rode o **`Instalar_Extras.bat`** se quiser criar atalho na Área de Trabalho e instalar microfone offline / importador de skins com 1 clique)*.
   * *Se o Windows mostrar o aviso azul do SmartScreen na 1ª vez:* clique em **Mais informações → Executar assim mesmo**.
   * *Se o Windows Defender bloquear (falso positivo comum em `.exe` indie sem certificado pago):* abra **Segurança do Windows → Proteção contra vírus e ameaças → Gerenciar configurações → Exclusões → Adicionar pasta** e selecione a pasta `Aster-Companion`. O código-fonte é 100% aberto aqui no GitHub para conferência.

### 2. Ativar a Inteligência Artificial (grátis, leva 1 minuto)
Clique no ícone de engrenagem (ou digite `/config`) → aba **Cérebro (IA)**:
* **Opção A — Na Nuvem (recomendado para a maioria dos PCs):**
  1. Em **Onde a IA vai rodar**, escolha **Na Nuvem**.
  2. Escolha **Groq** (crie uma chave grátis em [console.groq.com/keys](https://console.groq.com/keys) — **também libera o microfone sem precisar de Python!**) ou **Google Gemini** ([aistudio.google.com/apikey](https://aistudio.google.com/apikey)).
  3. Cole a API Key, clique em **Listar modelos** para escolher um modelo ativo da sua conta e clique em **Testar IA**.
* **Opção B — 100% Offline no seu PC (Ollama):**
  1. Instale o [Ollama](https://ollama.com/download) e rode no terminal: `ollama pull qwen3:4b-instruct` (ou `ollama pull gemma3:12b` para respostas mais profundas).
  2. Em **Cérebro (IA)**, deixe em **Ollama Local**, clique em **Atualizar modelos** e **Testar IA**.

### 3. Colocar seu nome, voz e skins (opcional)
* **Seu nome:** aba **Memória** → preencha **"Como a Aster deve te chamar"** e clique em **Salvar Configurações**.
* **Voz da Aster falando:** aba **Voz** → ative a voz, crie uma chave em [fish.audio](https://fish.audio), cole a **Chave Fish Audio** + **ID da Voz** e clique em **Testar Voz**.
* **Microfone (falar direto com `Alt + V` ou dizendo *"Aster..."*):**
  * Se você configurou **Groq** na aba **Cérebro (IA)**, basta apertar **`Alt + V`** (ou clicar no microfone) — já funciona direto sem instalar nada.
  * Para usar microfone **offline** sem Groq (ou o importador beta de skins), basta abrir o **`Instalar_Extras.bat`** que já vem na pasta e escolher a opção no menu.
* **Skins:** a **Skin Natalina** já vem inclusa. Para adicionar outras skins baixadas, clique em **Visual → Skins → Abrir pasta skins**, cole a pasta da skin lá dentro e clique em **Atualizar**.
* **Como atualizar no futuro (1 clique, sem perder nada):** a própria Aster avisa quando sair versão nova — basta abrir **Configurações** e clicar em **Atualizar agora** (ou em **Verificar atualizações**). Ela baixa, atualiza e reinicia sozinha preservando 100% do seu `.env`, `memory.json`, `docs/`, `.venv/` e todas as suas skins próprias.

Veja todos os comandos e atalhos em [COMANDOS.md](COMANDOS.md).

---

## Principais Recursos

### Interface & Desktop Companion
* **Overlay transparente sem bordas** (`alwaysOnTop`) com **click-through inteligente** nas áreas transparentes ao redor da personagem.
* **System Tray (bandeja do Windows):** acesso rápido para abrir chat, configurações, passeio automático, falas espontâneas, inicialização com o Windows, sempre no topo e mostrar/esconder.
* **Modo Tela Limpa + Atalho Global `Alt + A`:** pílula de chat flutuante com ícones SVG minimalistas que se recolhe automaticamente após inatividade.
* **Animações e física:** piscar aleatório, expressões dinâmicas, caminhada em 8 frames pela barra de tarefas, física de pêndulo por inércia ao arrastar, encaixe sentado na borda de janelas abertas, balanço rítmico no modo música e interações de headpat.

### Visual, Skins & Temas
* **Sistema de Skins (`skins/`):** criação de pastas em **Visual → Skins**, checklist visual (6 essenciais, 3 expressões e extras), miniaturas reais, suporte a `.png`/`.webp`/`.gif`, troca ao vivo sem recompilar (`/skin natalina`, `/skin padrao`) e fallback inteligente dentro da própria skin.
* **Skin Natalina inclusa (`skins/natalina/`):** conjunto completo com 20 sprites, incluindo caminhada em 8 frames.
* **Cores, Escala & Partículas (`Visual → Cores & Efeitos`):** 6 presets ao vivo (*Glass Gothic*, *Kinkou Emerald*, *Crimson Natal*, *Cyber Ice*, *Sakura Nocturne* e *Obsidian Mono*), slider de tamanho (`75%`–`130%`) e partículas sutis opcionais.

### IA Híbrida, Memória & RAG Local
* **Nuvem ou Local (texto + visão):** suporte a **Ollama** local (com auto-start silencioso e dropdown de modelos) e APIs **OpenAI-compatible** na nuvem (Gemini, Groq, OpenRouter, OpenAI) com streaming em tempo real e fallback automático.
* **Escalonamento remoto (Hermes):** delegação opcional de tarefas complexas via API compatível com OpenAI (`/hermes ...` ou *"pede pro Hermes..."*).
* **Memória persistente (`memory.json`):** salva, lista e apaga fatos e preferências entre sessões, além do nome configurável (`USER_NAME`).
* **RAG local (`docs/`):** busca em Rust com stemming PT-BR nos arquivos `.txt`, `.md`, `.json`, `.csv` e `.log` colocados na pasta `docs/`.
* **Comentários espontâneos:** comentários curtos ocasionais baseados no horário, janela ativa ou música tocando (pausados automaticamente em tela cheia).

### Automação no Windows 11, Jogos & Voz
* **Voz personalizada (Fish Audio TTS):** síntese frase por frase em Rust com baixa latência, lip-sync real via `AnalyserNode`, controle de volume (`0%`–`100%`) e botão de teste.
* **Microfone offline (STT):** captura a 16 kHz com VAD adaptativo, wake-word `"Aster"` e worker Python opcional com `faster-whisper` (`int8` CPU).
* **Detecção de música (SMTC) & Modo Jogo:** lê nativamente via WinRT o que está tocando no Windows, acompanha partidas de League of Legends ao vivo e reconhece jogos populares.
* **Pomodoro & Lembretes com voz:** timers por comando ou painel visual com alerta sonoro suave e aviso falado.
* **Ações nativas em Rust:** abertura de aplicativos e URLs (`ShellExecuteW`, incluindo atalhos como `spot` e `dc`), controle de mídia e volume via Win32 (`keybd_event`), leitura do clipboard (`arboard`), Spotify via UI Automation, WhatsApp, Discord DM e Discord Webhook.

---

## Stack

* **Backend & Desktop:** Rust, [Tauri v2](https://v2.tauri.app/), Win32 e WinRT (`user32`, `shell32`, `dwmapi`, `Windows.Media.Control`)
* **Frontend:** HTML5, CSS3 e JavaScript puro (WebView2 nativo do Windows 11)
* **Voz:** Fish Audio API (TTS em Rust) + Python 3.12 / `faster-whisper` (STT offline opcional)
* **IA:** [Ollama](https://ollama.com/) (local) + APIs OpenAI-compatible na Nuvem (Gemini, Groq, OpenRouter, OpenAI) + Hermes (opcional)

---

## Como Compilar Localmente (Desenvolvimento)

### Pré-requisitos
* [Node.js](https://nodejs.org/) (v18+)
* [Rust (`rustup`)](https://rustup.rs/) + Visual Studio C++ Build Tools
* *(Opcional, para IA offline)* [Ollama](https://ollama.com/) (`ollama pull qwen3:4b-instruct` e `ollama pull qwen2.5vl:3b` para visão)
* *(Opcional, para microfone offline e importador beta de skins)* Python 3.12

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
│   │   ├── brain.rs      # IA híbrida (Ollama + Cloud OpenAI-compatible), streaming, visão, teste de IA, Hermes e clipboard
│   │   ├── config.rs     # Leitura, cache em RAM e gravação segura do .env
│   │   ├── media.rs      # Detecção nativa de música no Windows (SMTC / WinRT)
│   │   ├── memory.rs     # Memória persistente (memory.json)
│   │   ├── docs.rs       # Busca local em documentos (RAG com stemming PT-BR)
│   │   ├── games.rs      # Modo Jogo (API ao vivo do LoL + detecção de jogos)
│   │   ├── skins.rs      # Listagem, criação, checklist, importador beta e troca de skins ao vivo
│   │   ├── updater.rs    # Verificação de versão no GitHub e auto-atualizador de 1 clique
│   │   └── voice.rs      # Groq Whisper (nuvem), faster-whisper (offline) e Fish Audio (TTS)
│   └── tauri.conf.json
├── Instalar_Extras.bat   # Assistente temático de 1 clique (atalho, microfone offline, importador)
├── pack_release.ps1      # Empacotador automático da Release (.zip organizado)
├── transcrever.py        # Worker offline de Speech-to-Text (faster-whisper)
├── processar_skin.py     # Importador/processador local de skins (Beta)
├── build_exe.ps1         # Script de build release -> Aster.exe
├── COMANDOS.md           # Guia completo de comandos e interações
├── CHANGELOG.md          # Histórico de versões
├── .env.example          # Template de configuração (sem chaves reais)
└── requirements.txt      # Dependências Python opcionais
```

---

## Privacidade

* A Aster Companion **não coleta telemetria, analytics nem envia dados para servidores próprios**.
* Todas as chaves de API (`.env`), memórias (`memory.json`), documentos (`docs/`) e skins ficam armazenados **100% localmente** no seu PC.
* Requisições externas ocorrem apenas para os serviços que o próprio usuário configurar (ex.: Groq, Google Gemini, OpenAI, Fish Audio, Discord Webhook) e para a verificação opcional de atualizações no GitHub Releases.

---

## Licença & Créditos

* **Código-fonte:** distribuído sob a [MIT License](LICENSE).
* **Arte e identidade visual:** os sprites originais e a identidade visual da personagem **Aster** são de uso pessoal / não comercial.
* Inspirado no projeto open-source [capri-desktop](https://github.com/lachydotmcg/capri-desktop).
