# Changelog

Todas as mudanças relevantes do projeto são documentadas neste arquivo.

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
