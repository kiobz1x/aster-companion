# Changelog

Todas as mudanças relevantes do projeto são documentadas neste arquivo.

## [1.3.7] - 2026-11-27

### Melhorias & Correções
- **Pomodoro & Timers com unidades flexíveis:** adicionado seletor `min / h / seg` ao lado do campo de tempo e parser inteligente que reconhece diretamente `2h`, `30s`, `15m`, `1h30m` ou horário fixo (`22:30`).
- **Apelidos da memória no Discord & WhatsApp:** ao pedir pra mandar mensagem usando um apelido salvo na Memória (ex.: *"João"* → nick fixo `"arrozefarinha"`), a Aster agora consulta suas memórias antes do atalho local.
- **Abrir sites/pedidos compostos sem erro `Win32:2`:** pedidos como *"abre o docs no vivaldi"*, *"abre o youtube"* ou qualquer item que não seja um programa/jogo instalado agora passam automaticamente pra IA abrir a URL certa.

---

## [1.3.6] - 2026-11-27

### Voz & Atalhos
- **Atalho global `Alt + V` (falar direto):** de qualquer tela ou jogo, aperta `Alt + V`, ouve um bip suave e fala o comando direto (sem clicar na Aster e sem precisar dizer `"Aster"`). Se o microfone não estava ligado antes, desliga automaticamente após o comando.
- **Mais tempo de pausa ao falar:** silêncio de corte aumentado de `0,75s` para `1,6s`, duração máxima da frase de `9s` para `15s` e janela de escuta para `16s`.
- **Reconhecimento de `"Aster"` aprimorado:** adicionado `prompt` de vocabulário no Groq Whisper e novas variações tolerantes na wake-word + bip suave ao ativar.

### Correções
- **Balanço no modo música (`musicGroove`) corrigido:** removida a classe `.bounce` após `450ms` (antes ficava presa e sobrescrevia a animação de música e respiração), impedido o modo sono (`sleepy`) durante reprodução de música e ajustado o balanço para ficar visivelmente mais fluido (`±2.2°`).

---

## [1.3.5] - 2026-11-27

### Novos Recursos
- **Auto-atualizador de 1 clique (`updater.rs`):** ao abrir a Aster, verifica automaticamente a última Release no GitHub, avisa no balão e exibe um banner nas Configurações com botão **"Atualizar agora"** — baixa, substitui os arquivos do app (preservando 100% do `.env`, `memory.json`, `docs/`, `.venv/` e skins próprias) e reinicia sozinha.
- **Abrir qualquer jogo instalado na Steam automaticamente:** ao pedir *"abre o Stardew Valley"*, *"abre o Elden Ring"*, etc., a Aster lê os manifestos da sua biblioteca Steam (em todos os HDs/SSDs via `libraryfolders.vdf`) e abre direto via `steam://rungameid/<ID>`.
- **Assistente `Instalar_Extras.bat` temático (roxo e preto) + pacote `.zip` organizado:** ao extrair o `.zip`, todos os arquivos já vêm dentro de `Aster-Companion/` com menu interativo de 1 clique para criar atalho na Área de Trabalho e instalar opcionais (Microfone Offline e Importador de Skins Beta) sem precisar digitar comandos.

---

## [1.3.3] - 2026-11-27

### Performance & Jogos
- **Modo Jogo Leve automático (`body.game-perf`):** ao detectar League of Legends, jogos conhecidos ou janelas em tela cheia, desativa `backdrop-filter: blur()`, `drop-shadow` do sprite, animação contínua `breathe`, partículas e tilt no `mousemove` para não competir com o FPS do jogo no DWM do Windows.
- **Lip-sync 90% mais leve:** trocado de `requestAnimationFrame` (60–144×/s) para intervalo de `120ms` (~8×/s) com o mesmo visual, reduzindo uso de GPU do WebView2 durante a fala.
- **Menos IPC durante a digitação:** `ResizeObserver` pausado enquanto a Aster escreve no balão.

### Correções
- **Ollama sob demanda:** `ensure_ollama_running()` não inicia mais o Ollama em segundo plano quando o motor escolhido é **Na Nuvem**.
- **Erro real da Nuvem preservado:** se a API na Nuvem falhar e o Ollama local não estiver ligado, exibe o erro real da Nuvem (ex.: `429` / `503`) em vez de mostrar *"Ollama desligado"*.

---

## [1.3.1] - 2026-11-25

### Novos Recursos & Melhorias
- **Fechar aplicativos (`[CLOSE_APP: nome]`):** agora a Aster fecha apps conhecidos por comando direto ou conversa (*"fecha o Spotify"*, *"fecha o Discord"*, *"fecha o navegador"*, *"fecha a calculadora"*, etc.) com whitelist segura.
- **Microfone sem Python (Groq Whisper na nuvem):** quem usa chave grátis do **Groq** em **Cérebro (IA)** agora pode usar o botão de microfone nativamente em Rust via `whisper-large-v3-turbo` (zero Python, zero `.venv`!). O Whisper offline continua disponível como alternativa.
- **Botão "Listar modelos" na API na Nuvem:** consulta automaticamente os modelos ativos da API Key (Groq, Gemini, OpenRouter ou OpenAI) e permite escolher em um dropdown, evitando erros 404 de modelos aposentados.
- **Onboarding e guias diretos nas Configurações:** links clicáveis para criar chaves grátis no Groq, Google AI Studio e Fish Audio, explicação clara de como instalar skins, e aviso amigável na 1ª abertura caso nenhuma IA esteja configurada.
- **Guia explícito para iniciantes no `README.md`:** passo a passo completo de download (só o `.zip`), configuração de IA, voz, microfone e skins.

### Correções
- **Gemini `400 Bad Request` corrigido:** `reasoning_effort: "none"` agora só é enviado para `gemini-2.5-flash` (não-lite), liberando `gemini-3.5-flash-lite` e demais modelos.
- **Presets atualizados:** Google Gemini (`gemini-3.5-flash-lite`) e Groq (`llama-3.1-8b-instant`).

---

## [1.3.0] - 2026-11-24

### Novos Recursos
- **Motor de IA híbrido (Ollama local ou API na Nuvem):** suporte a provedores OpenAI-compatible com presets prontos para **Google Gemini**, **Groq**, **OpenRouter** e **OpenAI** (com streaming SSE, visão de tela na nuvem, desativação de *thinking* no Gemini 2.5 para respostas rápidas e fallback automático para o Ollama).
- **Botão "Testar IA":** valida a conexão com a IA selecionada (Nuvem ou Ollama) direto na aba **Cérebro (IA)**, exibindo o modelo e a latência em `ms`.
- **Nome configurável (`USER_NAME`):** campo **"Como a Aster deve te chamar"** em **Memória**, aplicado dinamicamente ao prompt, saudações, lembretes, reações de jogos e teste de voz.
- **Modo Jogo ao vivo (`games.rs`):** integração nativa com a API local do **League of Legends** (First Blood, Multikills, Dragão, Barão, Vitória/Derrota) e detecção de jogos populares (Valorant, CS2, Minecraft, Elden Ring, GTA V, Fortnite, entre outros).
- **Painel visual de Pomodoro & Timers:** iniciar ciclos 25/5 ou 50/10, criar lembretes com voz e acompanhar contagens regressivas ao vivo em **Integrações → Pomodoro & Timers**.
- **Controle de tamanho da Aster na tela:** slider de escala (`75%` a `130%`) em tempo real em **Visual → Cores & Efeitos**.
- **Animação rítmica no modo música (`musicGroove`):** balanço suave enquanto a Aster está com o sprite de fone (`music`).
- **Suporte a sprites animados (`.webp` e `.gif`)** e miniaturas reais nos cards de skin.
- **Guia completo de comandos (`COMANDOS.md`):** referência de todos os comandos slash, frases naturais, atalhos e interações.

### Voz, Contexto & Interface
- **Voz mais rápida durante streaming:** síntese Fish Audio disparada frase por frase em paralelo enquanto o texto ainda está sendo gerado (`normalize: false`, `chunk_length: 150`).
- **Contexto e RAG aprimorados:** janela de contexto expandida (`num_ctx: 4096`, histórico de 18 mensagens) e stemming em PT-BR para busca local em `docs/`.
- **Redesenho das Configurações:** janela redimensionável (`760×600`) reorganizada em 5 abas (**Cérebro (IA)**, **Voz**, **Visual**, **Integrações** e **Memória**), bolinhas de status inteligentes e indicador de alterações não salvas.

### Otimizações & Correções
- **Sprites otimizados para `720px` (2× Retina):** redução de ~67% no tamanho dos PNGs sem perda visual na tela, e padrão do importador `processar_skin.py` atualizado para `720px`.
- **Importador de skins (Beta):** defringe aprimorado na borda do `rembg`, compensação de contraste/saturação para frames extraídos de `.mp4` e correção de race condition nos `Blob` URLs durante trocas rápidas de skin.

---

## [1.2.0] - 2026-11-23

### Novos Recursos
- **Detecção real de música (Windows SMTC):** lê nativamente via WinRT o que está tocando no Spotify, YouTube ou navegador, ativa automaticamente o sprite `music` ao dar play, retorna ao `idle` ao pausar e permite perguntas/comentários sobre a faixa atual.
- **Consciência da skin ativa:** a Aster reconhece qual skin está vestindo ao conversar.
- **Streaming em tempo real + Auto-start do Ollama:** respostas aparecem token por token no balão (filtrando tags internas) e o servidor local `ollama serve` é iniciado silenciosamente em segundo plano caso esteja desligado.
- **Visão de tela sob demanda (`/ver`, `/tela`, *"olha minha tela"*):** captura um print da tela principal em memória (sem arquivo temporário) e analisa com o modelo multimodal configurado no Ollama.
- **Lembretes, timers e Pomodoro com voz:** suporte a `/pomodoro`, `/lembrete`, `/timer`, `/lembretes`, `/cancelar-lembretes` e frases naturais, com alerta sonoro suave (Web Audio API) e aviso falado.
- **Dropdown automático de modelos do Ollama:** seleção direta dos modelos instalados no PC para chat e visão em **IA & Conexões**.
- **Partículas sutis opcionais:** efeitos leves de fundo (flocos na skin natalina, pétalas no tema Sakura ou brilhos suaves) com toggle ao vivo em **Temas**.
- **Importador de skins (Beta):** processamento local via `processar_skin.py` (`rembg / isnet-anime`), contorno preto opcional de 2px e extração automática de ciclo de caminhada a partir de `.mp4`.

### Otimizações
- **Clientes HTTP persistentes (`OnceLock`)** em `brain.rs` para reaproveitar conexões keep-alive com Ollama, Hermes e webhooks.
- **Cache em RAM do `.env` (`RwLock`)** em `config.rs`, eliminando leituras repetidas de disco.
- **Skins com `Blob` URLs + `revokeObjectURL`** em `app.js`, reduzindo picos de memória ao alternar skins.
- **Throttle com `requestAnimationFrame` e diff em `syncHitRects()`**, cortando ~90% das chamadas IPC redundantes.
- **Pausa automática quando oculta** (`document.hidden`) para partículas e polling de música.

### Ajustes & Limpeza
- Padronização de toda a interface e backend para **Hermes**.
- Revisão de textos nas abas **Docs (RAG)**, **Skins**, **Temas** e **IA & Conexões**.
- Limpeza geral de comentários no código-fonte.

---

## [1.1.0] - 2026-11-22

- Síntese de voz personalizada via **Fish Audio** em Rust com lip-sync real e controle de volume (0–100%).
- Sistema de **Skins ao vivo** (`skins/`) com checklist visual, fallback interno e **Skin Natalina** inclusa.
- **6 presets de cores** em tempo real na aba **Temas**.
- Redesenho **Glass Gothic** das configurações com tipografia `Inter` e ícones SVG minimalistas.
- Correção da física de pêndulo ao arrastar e manutenção da pose sentada enquanto fala na borda da janela.

---

## [1.0.0] - 2026-11-20

- Migração completa para **Tauri v2 (Rust + HTML/CSS/JS)** com overlay transparente, click-through inteligente, System Tray, hotkey `Alt + A`, caminhada em 8 frames, janela-perch, memória persistente, RAG local (`docs/`), automação nativa no Windows 11 e microfone offline (`faster-whisper`).
