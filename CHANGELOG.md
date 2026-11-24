# Changelog

Todas as mudanças relevantes do projeto são documentadas neste arquivo.

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
