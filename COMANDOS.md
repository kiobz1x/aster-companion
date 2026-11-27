# Guia de Comandos — Aster Companion

Referência completa de todos os comandos slash (`/`), comandos por frase natural (texto ou voz), atalhos e interações disponíveis na Aster Companion.

---

## Atalhos e Interações Diretas

| Ação | Como fazer | O que faz |
| :--- | :--- | :--- |
| Abrir / focar chat | `Alt + A` (global) ou clicar na Aster | Abre a pílula de chat de qualquer janela do Windows |
| Enviar / fechar chat | `Enter` / `Esc` | Envia a mensagem digitada ou recolhe a barra de chat |
| Mover / sentar na janela | Arrastar a Aster com o mouse | Move pela tela com física de pêndulo; soltar perto do topo de uma janela aberta faz ela sentar na borda |
| Chacoalhar | Sacudir rápido enquanto arrasta | Reação irritada (`angry`) com fala |
| Carinho (*headpat*) | Passar o mouse repetidamente no topo da cabeça | Solta partículas e uma resposta aleatória |
| Comando por voz | Ativar microfone e dizer `"Aster ..."` | Transcreve offline (Whisper) e executa qualquer comando abaixo |
| Bandeja do Windows | Clique direito no ícone da Aster (System Tray) | Acesso rápido a Chat, Configurações, Passeio automático, Falas espontâneas, Iniciar com o Windows, Sempre no topo, Mostrar/Esconder e Sair |

---

## Sistema, Configurações e Skins

| Comando | Alternativas naturais | O que faz |
| :--- | :--- | :--- |
| `/config` ou `/settings` | *"abre as configurações da aster"* | Abre o painel de Configurações |
| `/skin <nome>` | *"coloca a skin natalina"*, *"volta pra skin padrão"* | Troca a skin ativa ao vivo (`padrao`, `natalina` ou qualquer pasta em `skins/`) |
| `/skins` | *"abre a pasta skins"* | Abre a pasta `skins/` no Explorador de Arquivos |
| `/docs` | *"abre a pasta docs"* | Abre a pasta `docs/` onde ficam os arquivos de apoio (RAG local) |

---

## Visão de Tela e Área de Transferência

| Comando | Alternativas naturais | O que faz |
| :--- | :--- | :--- |
| `/ver [pergunta]` | `/tela [pergunta]`, *"olha minha tela..."*, *"o que tem na minha tela?"* | Captura um print em memória da tela principal e analisa com a IA (Nuvem ou Ollama) |
| Leitura de `Ctrl+C` | Mencionar *"copiei"*, *"ctrl+c"* ou *"clipboard"* na pergunta | Anexa automaticamente o texto da sua área de transferência ao contexto da resposta |

---

## Lembretes, Timers e Pomodoro (com voz)

> Também podem ser gerenciados visualmente em **Configurações → Integrações → Pomodoro & Timers**.

| Comando | Alternativas naturais | O que faz |
| :--- | :--- | :--- |
| `/pomodoro` | *"inicia um pomodoro"* | Inicia um ciclo Pomodoro padrão de 25 min de foco e 5 min de pausa |
| `/pomodoro <foco> <pausa>` | *"inicia um pomodoro de 50 min"* | Inicia Pomodoro personalizado (ex.: `/pomodoro 50 10`) |
| `/lembrete <tempo> <texto>` | `/timer 15m ...`, *"me lembra em 20 minutos de beber água"* | Agenda um lembrete relativo usando `s` (segundos), `m` (minutos) ou `h` (horas) |
| Horário fixo | *"me lembra às 22:30 de dormir"*, *"me avisa às 18h da reunião"* | Agenda um lembrete para o horário exato informado |
| `/lembretes` ou `/timers` | *"meus lembretes"*, *"quais são os timers?"* | Lista todos os lembretes e Pomodoros ativos com o tempo restante |
| `/cancelar-pomodoro` | *"cancela o pomodoro"* | Cancela apenas o Pomodoro em andamento |
| `/cancelar-lembretes` | `/cancelar-timers`, *"cancela os lembretes"* | Cancela todos os lembretes e timers ativos |

---

## Memória Persistente

> Os fatos ficam salvos em `memory.json` e também podem ser editados em **Configurações → Memória**.

| Comando | Alternativas naturais | O que faz |
| :--- | :--- | :--- |
| `/lembrar <fato>` | *"lembra que minha banda favorita é..."*, *"anota que..."* | Salva um fato fixo na memória permanente da Aster |
| `/memoria` | *"o que você lembra de mim?"*, *"mostra suas memórias"* | Lista todos os fatos salvos atualmente |
| `/esquecer <termo ou número>` | *"esquece que..."*, *"apaga da memória..."* | Remove memórias que contenham o termo informado (ou pelo índice da lista) |
| `/limpar-memoria` | — | Apaga todas as memórias salvas |

---

## Animações, Passeio e Falas Espontâneas

| Comando | Alternativas naturais | O que faz |
| :--- | :--- | :--- |
| `/walk` | *"anda aí"*, *"vai passear"*, *"dá uma volta"* | Faz a Aster caminhar pela barra de tarefas |
| *"para de andar"* | *"fica quieta"*, *"fica parada"* | Pausa o passeio automático |
| *"pode andar"* | *"pode passear"*, *"volta a andar"* | Reativa o passeio automático |
| `/sit` | *"senta aí"*, *"senta na janela"* | Senta na borda superior de uma janela aberta (ou no lugar atual) |
| *"levanta"* | *"desce"*, *"fica em pé"* | Faz a Aster descer de volta para a barra de tarefas |
| `/mood <nome>` | — | Troca a expressão manualmente (`idle`, `smug`, `angry`, `sleepy`, `think`, `music`, `listen`, `drag`, `sit`, `blink`) |
| `/comentar` | — | Dispara imediatamente um comentário espontâneo (janela ativa, curiosidade ou horário) |
| *"modo silencioso"* | *"para de falar sozinha"*, *"fica calada"* | Desativa comentários espontâneos (ela só fala quando chamada) |
| *"pode falar sozinha"* | *"volta a comentar"* | Reativa os comentários espontâneos |

---

## Mídia, Spotify e Aplicativos (Windows 11)

| Comando / Frase | Exemplos | O que faz |
| :--- | :--- | :--- |
| Tocar no Spotify | *"toca Linkin Park no spotify"*, *"bota lo-fi no spot"* | Abre o Spotify e inicia a reprodução da busca |
| Play / Pause | *"pausa"*, *"despausa"*, *"play"*, *"para a música"* | Pausa ou retoma a mídia atual do Windows |
| Trocar faixa | *"próxima"*, *"pula"*, *"volta"*, *"música anterior"* | Avança ou volta a faixa atual |
| Volume | *"muta"*, *"silencia"*, *"aumenta o volume"*, *"abaixa o volume"* | Controla o volume e o mudo do sistema via Win32 |
| Abrir aplicativo ou jogo Steam | *"abre o brave"*, *"abre o discord"*, *"abre o stardew valley"*, *"abre o elden ring"* | Abre aplicativos instalados no Windows (aceita `spot`/`dc`) e **qualquer jogo instalado na sua Steam** automaticamente |
| Fechar aplicativo | *"fecha o spotify"*, *"fecha o discord"*, *"fecha o navegador"*, *"fecha a calculadora"* | Encerra aplicativos conhecidos abertos no Windows de forma segura |
| Abrir site / busca | *"pesquisa no youtube como configurar docker"*, *"abre o github"* | Abre URLs ou buscas diretamente no navegador padrão (via IA) |

---

## Mensagens e Integrações Externas

| Comando | Alternativas naturais | O que faz |
| :--- | :--- | :--- |
| WhatsApp | *"manda pro João no whatsapp dizendo cheguei"* (`wpp` / `zap`) | Abre a conversa no WhatsApp e envia a mensagem |
| Discord DM | *"manda pro Alex no discord dizendo bora duo"* (`dc`) | Abre a DM no Discord e envia a mensagem (preservando seu `Ctrl+C`) |
| `/discord <mensagem>` | *"posta no canal do discord que a live começou"* | Envia a mensagem para o canal via Discord Webhook (se configurado) |
| `/hermes <tarefa>` | *"pede pro hermes refatorar..."* | Delega tarefas pesadas (código, auditoria, pesquisa profunda) ao agente Hermes (opcional) |

---

## Recursos Automáticos (sem precisar de comando)

- **Detecção de música (Windows SMTC):** quando uma música toca no Spotify, YouTube ou navegador, a Aster coloca o fone (`music`) com animação rítmica e pode comentar a faixa ocasionalmente; ao pausar, volta para `idle`.
- **Modo Jogo:** reconhece jogos abertos (Valorant, CS2, Minecraft, Elden Ring, GTA V, Fortnite, entre outros) e acompanha partidas de **League of Legends** ao vivo (First Blood, Multikills, Dragão, Barão, Vitória/Derrota). Pode ser ligado/desligado em **Configurações → Integrações**.
- **Arquivos de apoio (`docs/`):** qualquer arquivo `.txt`, `.md`, `.json`, `.csv` ou `.log` colocado na pasta `docs/` é consultado automaticamente quando você pergunta sobre o assunto.
