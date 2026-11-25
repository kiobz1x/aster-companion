<div align="center">

<img width="872" height="720" alt="aster_actions" src="https://github.com/user-attachments/assets/b61fc976-78a8-49bf-b3e6-d58c374c733a" />

# Aster Companion

Assistente virtual de desktop para **Windows 11**, construída em **Tauri v2 (Rust + HTML/CSS/JS)** com foco em baixo consumo de recursos, animações interativas estilo *desktop pet* e automação nativa do sistema com IA híbrida.

<p>
  <a href="https://github.com/kiobz1x/aster-companion/releases"><img src="https://img.shields.io/badge/versão-v1.3.0-7c3aed?style=flat-square" alt="Versão v1.3.0" /></a>
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

## Novidades da v1.3.0

> [!TIP]
> **Destaque da versão:** a Aster agora roda tanto **100% offline no Ollama** quanto **na nuvem (Gemini, Groq, OpenRouter ou OpenAI)** com fallback automático — basta colar sua API Key e testar direto nas Configurações, sem precisar de placa de vídeo dedicada.

| Novidade | O que muda na prática |
| :--- | :--- |
| **IA Híbrida + Botão Testar IA** | Escolha entre **Ollama local** ou **API na Nuvem** (presets prontos para Gemini, Groq, OpenRouter e OpenAI) com streaming SSE, visão de tela e teste de conexão com latência em `ms`. |
| **Voz frase por frase** | A síntese do Fish Audio dispara em paralelo assim que cada frase fecha no streaming, cortando a espera até a primeira fala. |
| **Modo Jogo ao vivo** | Acompanha partidas de **League of Legends** em tempo real (First Blood, Multikills, Dragão, Barão, Vitória/Derrota) e reconhece jogos populares (Valorant, CS2, Minecraft, Elden Ring, GTA V, etc.). |
| **Pomodoro & Timers visuais** | Painel ao vivo em **Integrações → Pomodoro & Timers** para iniciar ciclos `25/5` ou `50/10`, criar lembretes com voz e acompanhar a contagem regressiva. |
| **Personalização completa** | Defina **como a Aster deve te chamar** (`USER_NAME`), ajuste o **tamanho dela na tela** (`75%`–`130%`), curta o balanço rítmico no modo música e use sprites animados (`.webp`/`.gif`). |
| **Configurações redesenhadas** | Janela redimensionável (`760×600`) em 5 abas (**Cérebro (IA)**, **Voz**, **Visual**, **Integrações** e **Memória**) com bolinhas de status inteligentes e aviso de alterações não salvas. |

<details>
<summary><strong>Ver todas as melhorias técnicas da v1.3.0</strong></summary>

* **Sprites otimizados para `720px` (2× Retina):** redução de ~67% no tamanho dos arquivos PNG sem perda visual na tela, com padrão atualizado também no `processar_skin.py`.
* **Contexto e busca local aprimorados:** janela de contexto expandida (`num_ctx: 4096`, histórico de 18 mensagens) e stemming em PT-BR no RAG local (`docs/`).
* **Gemini 2.5 sem atraso de raciocínio:** envio automático de `reasoning_effort: "none"` para respostas rápidas no `gemini-2.5-flash`.
* **Importador de skins (Beta):** defringe aprimorado na borda do `rembg`, compensação de contraste/saturação para frames de `.mp4` e proteção contra race condition nos `Blob` URLs.
* **Guia completo de uso:** todos os comandos, frases naturais e atalhos documentados em [COMANDOS.md](COMANDOS.md).

</details>

---

## Download Rápido (Windows 11)

Para apenas usar a Aster **sem instalar Node, Rust ou Python**:

1. Baixe a versão mais recente na aba [Releases](https://github.com/kiobz1x/aster-companion/releases) e extraia a pasta no seu PC.
2. Execute `Aster.exe`.
3. Clique no ícone de engrenagem (ou digite `/config`):
   * Em **Cérebro (IA)**, escolha **Na Nuvem** (ex.: Google Gemini ou Groq) e cole sua API Key — ou deixe em **Ollama Local** se já tiver o Ollama instalado. Clique em **Testar IA**.
   * Em **Memória**, preencha **"Como a Aster deve te chamar"**.
4. Aperte `Alt + A` ou clique na Aster para conversar. Veja tudo que ela faz em [COMANDOS.md](COMANDOS.md).

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
│   │   └── voice.rs      # Worker Python (faster-whisper STT) e síntese Fish Audio (TTS)
│   └── tauri.conf.json
├── transcrever.py        # Worker offline de Speech-to-Text (faster-whisper)
├── processar_skin.py     # Importador/processador local de skins (Beta)
├── build_exe.ps1         # Script de build release -> Aster.exe
├── COMANDOS.md           # Guia completo de comandos e interações
├── CHANGELOG.md          # Histórico de versões
├── .env.example          # Template de configuração (sem chaves reais)
└── requirements.txt      # Dependências Python opcionais
```

---

## Licença & Créditos

* Código-fonte inspirado em [capri-desktop](https://github.com/lachydotmcg/capri-desktop).
* **Uso pessoal / portfólio:** fique à vontade para estudar e adaptar o código; os sprites e a identidade visual da personagem **Aster** são de uso pessoal.
