# ✦ Aster Companion

Assistente virtual de desktop para **Windows 11**, construída em **Tauri v2 (Rust + HTML/CSS/JS)** com foco em leveza, animações interativas estilo *desktop pet* e automação real do sistema com IA híbrida.

> Inspirado no projeto open-source [capri-desktop](https://github.com/lachydotmcg/capri-desktop).

---

## ✨ Principais Recursos

### 🖥️ Experiência de Desktop Companion
* **Overlay transparente sem bordas** (`alwaysOnTop`, sem sombra retangular do Windows) com **click-through inteligente** nas áreas transparentes ao redor da personagem.
* **System Tray (bandeja do Windows)** com menu rápido (*Abrir Chat*, *Configurações & Memória*, *Passeio Automático*, *Iniciar com o Windows*, *Sempre no Topo*, *Mostrar/Esconder*).
* **Modo Tela Limpa + Atalho Global `Alt + A`:** balão e chat recolhem automaticamente após alguns segundos, deixando apenas a Aster flutuando na tela.
* **Animações e Física:**
  * Piscar natural aleatório, expressões dinâmicas (`idle`, `smug`, `angry`, `sleepy`, `think`, `listen`, `music`, `talk_calm`, `talk`).
  * **Caminhada em 8 frames** com passeio autônomo pela barra de tarefas (com espelhamento automático esquerda/direita).
  * **Física de pêndulo ao arrastar:** segurada pela gola do suéter com balanço elástico real seguindo a velocidade do mouse (Win32 `GetCursorPos` + `GetAsyncKeyState`).
  * **Sentar nas janelas abertas:** ao soltar perto da borda superior de uma janela — ou por vontade própria — ela encaixa sentada em cima da guia e desce automaticamente se a janela fechar/minimizar.
  * **Headpat / Carinho:** passar o mouse na cabeça dela solta partículas roxas e reações curtas.

### 🧠 IA Híbrida, Memória e RAG Local
* **Cérebro Local ([Ollama](https://ollama.com/)):** respostas rápidas no estilo da personagem (ex: `qwen3:4b-instruct`) com emissão de tags de humor e ação.
* **Escalonamento Remoto (Kaida / Hermes):** delega tarefas pesadas de código ou pesquisa para uma API compatível com OpenAI (`/kaida ...` ou *"peça para a Kaida..."*).
* **Memória Persistente (`memory.json`):** salva, lista e apaga fatos e preferências entre sessões (por chat, voz ou painel visual).
* **Base de Documentos Locais (`docs/` — RAG):** busca instantânea em Rust nos arquivos `.txt`, `.md`, `.json`, `.csv` e `.log` colocados em `docs/`, injetando trechos relevantes no contexto automaticamente.
* **Comentários espontâneos:** comentários curtos ocasionais baseados no horário e na janela ativa (com detecção de **tela cheia** para nunca atrapalhar jogos ou vídeos).

### ⚡ Automação no Windows 11 e Voz (STT)
* **Ações nativas em Rust:** abertura segura de aplicativos e URLs (`ShellExecuteW`), controle de mídia e volume em <1 ms via Win32 (`keybd_event`), leitura nativa do `Ctrl+C` (`arboard`).
* **Integrações:** busca e reprodução no **Spotify** via Windows UI Automation, envio de mensagens no **WhatsApp** e **Discord DM** preservando o clipboard, e postagem em **Discord Webhook**.
* **Escuta por voz offline (STT):** captura a 16 kHz com VAD adaptativo, wake-word `"Aster"` e worker persistente em Python com `faster-whisper` (`int8` na CPU), gerenciado e encerrado limpinho pelo Rust.
* **Painel visual de Configurações (`settings.html`):** edita `.env`, modelo do Whisper, memórias e documentos `docs/` pela própria interface.

---

## 🛠️ Stack

* **Desktop & Backend:** Rust + [Tauri v2](https://v2.tauri.app/) + Win32 APIs (`user32`, `shell32`, `dwmapi`)
* **Frontend:** HTML5, CSS3 e JavaScript puro (WebView2 nativo do Windows 11)
* **STT (opcional):** Python 3.12 + `faster-whisper` (`int8` CPU)
* **IA:** Ollama (local) + API OpenAI-compatible (Hermes / Kaida)

---

## 🚀 Como Rodar Localmente (Windows 11)

### Pré-requisitos
* [Node.js](https://nodejs.org/) (v18+)
* [Rust (`rustup`)](https://rustup.rs/) + Visual Studio C++ Build Tools
* [Ollama](https://ollama.com/) rodando localmente (`ollama pull qwen3:4b-instruct`)
* *(Opcional, para microfone)* Python 3.12

### Instalação
1. Clone o repositório e instale o CLI do Tauri:
   ```powershell
   npm install
   ```
2. Copie o arquivo de exemplo e preencha suas configurações locais:
   ```powershell
   Copy-Item .env.example .env
   ```
3. *(Opcional — para usar o botão de microfone 🎤)* Crie o ambiente virtual Python e instale o Whisper:
   ```powershell
   python -m venv .venv
   .\.venv\Scripts\pip install -r requirements.txt
   ```
4. Rode em modo desenvolvimento ou gere o executável `Aster.exe`:
   ```powershell
   npm run dev        # desenvolvimento
   npm run build:exe  # compila o Aster.exe otimizado na raiz
   ```

---

## 📁 Estrutura do Projeto

```text
Aster-Companion/
├── src/                  # Interface WebView2 (index.html, app.js, settings.html, sprites/)
├── src-tauri/
│   ├── src/
│   │   ├── lib.rs        # Janela, click-through, System Tray, hotkey Alt+A, drag físico e perch
│   │   ├── actions.rs    # Win32 mídia, ShellExecuteW, Spotify UIA, WhatsApp e Discord DM
│   │   ├── brain.rs      # Clientes HTTP (Ollama, Hermes, Discord Webhook) e clipboard
│   │   ├── config.rs     # Leitura e gravação segura do .env
│   │   ├── memory.rs     # Memória persistente (memory.json)
│   │   ├── docs.rs       # Busca local em documentos (RAG em docs/)
│   │   └── voice.rs      # Gerenciamento do worker Python (faster-whisper)
│   └── tauri.conf.json
├── transcrever.py        # Worker offline de Speech-to-Text (faster-whisper)
├── build_exe.ps1         # Script de build release -> Aster.exe
├── .env.example          # Template de configuração
└── requirements.txt      # Dependências Python do STT
```

---

## 📄 Licença & Créditos

* Código-fonte inspirado em [capri-desktop](https://github.com/lachydotmcg/capri-desktop).
* **Uso pessoal / portfólio:** fique à vontade para estudar e adaptar o código; os sprites e a identidade visual da personagem **Aster** são de uso pessoal.
