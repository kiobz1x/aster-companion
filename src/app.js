// Aster Companion - frontend principal
const invoke = window.__TAURI__?.core?.invoke;
const tauriWindow = window.__TAURI__?.window?.getCurrentWindow?.();
const tauriEvent = window.__TAURI__?.event;

const speechText = document.getElementById('speech-text');
const speechBubble = document.getElementById('speech-bubble');
const userInput = document.getElementById('user-input');
const sendBtn = document.getElementById('send-btn');
const micBtn = document.getElementById('mic-btn');
const cfgBtn = document.getElementById('cfg-btn');
const micIndicator = document.getElementById('mic-indicator');
const statusPill = document.getElementById('status-pill');
const modeLabel = document.getElementById('mode-label');
const dragHeader = document.getElementById('drag-header');
const wrapper = document.getElementById('aster-wrapper');
const sprite = document.getElementById('aster-sprite');
const inputBox = document.getElementById('input-box');

const DEFAULT_SPRITES = {
  idle:      'sprites/aster_idle.png',
  blink:     'sprites/aster_blink.png',
  talk:      'sprites/aster_talk.png',
  talk_calm: 'sprites/aster_talk_calm.png',
  smug:      'sprites/aster_smug.png',
  angry:     'sprites/aster_angry.png',
  sleepy:    'sprites/aster_sleepy.png',
  think:     'sprites/aster_think.png',
  drag:      'sprites/aster_drag.png',
  sit:       'sprites/aster_sit.png',
  listen:    'sprites/aster_listen.png',
  music:     'sprites/aster_music.png',
  walk1:     'sprites/aster_walk_1.png',
  walk2:     'sprites/aster_walk_2.png',
  walk3:     'sprites/aster_walk_3.png',
  walk4:     'sprites/aster_walk_4.png',
  walk5:     'sprites/aster_walk_5.png',
  walk6:     'sprites/aster_walk_6.png',
  walk7:     'sprites/aster_walk_7.png',
  walk8:     'sprites/aster_walk_8.png',
};

const SPRITES = { ...DEFAULT_SPRITES };

let WALK_FRAMES = [
  SPRITES.walk1,
  SPRITES.walk2,
  SPRITES.walk3,
  SPRITES.walk4,
  SPRITES.walk5,
  SPRITES.walk6,
  SPRITES.walk7,
  SPRITES.walk8,
];

function rebuildWalkFrames() {
  WALK_FRAMES = [
    SPRITES.walk1,
    SPRITES.walk2,
    SPRITES.walk3,
    SPRITES.walk4,
    SPRITES.walk5,
    SPRITES.walk6,
    SPRITES.walk7,
    SPRITES.walk8,
  ];
}

function preloadSprites() {
  Object.values(SPRITES).forEach((src) => {
    const img = new Image();
    img.src = src;
  });
}

let currentSkinId = 'padrao';
let userName = '';
let nowPlaying = { playing: false, title: '', artist: '', app: '' };
let lastTrackKey = '';
let lastMusicCommentAt = 0;
let lolEnabled = true;
let lolState = { in_game: false, summoner: '', champion: '', kills: 0, deaths: 0, assists: 0 };
let activeGameName = '';
let announcedGames = new Set();
let lastLolEventId = -1;
let lastLolCommentAt = 0;
let activeSkinBlobUrls = [];
let skinLoadSeq = 0;

function dataUriToBlobUrl(dataUri, bucket) {
  if (!dataUri || !dataUri.startsWith('data:')) return dataUri;
  const comma = dataUri.indexOf(',');
  if (comma === -1) return dataUri;
  const header = dataUri.slice(5, comma);
  const mime = header.split(';')[0] || 'image/png';
  const bin = atob(dataUri.slice(comma + 1));
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  const url = URL.createObjectURL(new Blob([bytes], { type: mime }));
  bucket.push(url);
  return url;
}

async function applySkin(skinId) {
  const seq = ++skinLoadSeq;
  const newBlobs = [];
  const nextSprites = { ...DEFAULT_SPRITES };

  if (invoke) {
    try {
      let targetSkin = currentSkinId;
      if (typeof skinId === 'string' && skinId.trim()) {
        targetSkin = skinId.trim();
      } else {
        const cfg = await invoke('get_settings').catch(() => null);
        targetSkin = (cfg?.active_skin || 'padrao').trim();
      }
      const rawOverrides = (await invoke('skins_get_overrides', { skinId: targetSkin || null })) || {};
      // Se outra chamada mais recente começou durante o await, aborta esta sem revogar nada
      if (seq !== skinLoadSeq) return;
      currentSkinId = targetSkin;

      const overrides = {};
      for (const [key, dataUri] of Object.entries(rawOverrides)) {
        if (key in nextSprites && dataUri) {
          const blobUrl = dataUriToBlobUrl(dataUri, newBlobs);
          overrides[key] = blobUrl;
          nextSprites[key] = blobUrl;
        }
      }
      const hasAny = Object.keys(overrides).length > 0;
      if (hasAny) {
        const skinBase = overrides.idle || Object.values(overrides)[0];
        if (!overrides.talk && overrides.talk_calm) nextSprites.talk = overrides.talk_calm;
        if (!overrides.talk_calm && overrides.talk) nextSprites.talk_calm = overrides.talk;
        if (!overrides.talk && !overrides.talk_calm) {
          nextSprites.talk = skinBase;
          nextSprites.talk_calm = skinBase;
        }
        if (!overrides.listen) nextSprites.listen = overrides.think || skinBase;
        if (!overrides.sleepy) nextSprites.sleepy = overrides.blink || skinBase;
        if (!overrides.blink) nextSprites.blink = overrides.sleepy || skinBase;
        if (!overrides.smug) nextSprites.smug = skinBase;
        if (!overrides.angry) nextSprites.angry = skinBase;
        if (!overrides.music) nextSprites.music = overrides.smug || skinBase;
        if (!overrides.think) nextSprites.think = skinBase;
        if (!overrides.drag) nextSprites.drag = skinBase;
        if (!overrides.sit) nextSprites.sit = skinBase;
      }
    } catch (e) {
      console.warn('[applySkin]', e);
    }
  }

  if (seq !== skinLoadSeq) {
    newBlobs.forEach((u) => { try { URL.revokeObjectURL(u); } catch (_) {} });
    return;
  }

  const oldBlobs = activeSkinBlobUrls;
  activeSkinBlobUrls = newBlobs;
  Object.assign(SPRITES, nextSprites);
  rebuildWalkFrames();
  preloadSprites();
  if (sprite && !isWalking) {
    sprite.src = SPRITES[currentMood] || SPRITES.idle;
  }
  // Só revoga os Blob URLs antigos depois que o DOM já aponta pros novos
  setTimeout(() => {
    oldBlobs.forEach((u) => { try { URL.revokeObjectURL(u); } catch (_) {} });
  }, 1500);
}

preloadSprites();

function ownerLabel() {
  return userName || 'usuário';
}

function ownerOf() {
  return userName ? `de ${userName}` : 'do usuário';
}

function vocative(prefix = ', ') {
  return userName ? `${prefix}${userName}` : '';
}

function buildBaseSystemPrompt() {
  const nameHint = userName
    ? `Chame a pessoa de "${userName}" naturalmente quando fizer sentido.`
    : 'Não invente um nome para a pessoa; trate-a de forma natural em segunda pessoa ("você").';
  const sampleUser = userName || 'Usuário';
  return `Você é Aster, assistente pessoal de desktop ${ownerOf()} no Windows 11.
Você TEM controle total do PC ${ownerOf()} através das tags de ação abaixo. Nunca diga que não consegue fazer algo no PC.
Personalidade: direta, humor seco, informal em PT-BR e respostas curtas (1 a 2 frases). ${nameHint}

Toda resposta DEVE começar com [MOOD: idle], [MOOD: smug] ou [MOOD: angry].

Tags disponíveis:
- [OPEN_APP: nome] -> brave, vivaldi, discord, spotify, whatsapp, vscode, steam, calculadora, notepad, explorador, terminal, configuracoes
- [OPEN_URL: https://...] -> abre qualquer site ou busca no navegador
- [SPOTIFY_PLAY: nome da musica ou artista] -> abre o Spotify e toca a música
- [WPP: contato | mensagem] -> envia mensagem no WhatsApp
- [DISCORD_DM: usuario | mensagem] -> envia DM no Discord
- [MEDIA: playpause], [MEDIA: next], [MEDIA: prev], [MEDIA: mute], [MEDIA: volup], [MEDIA: voldown] -> controla reprodução e volume
- [REMEMBER: fato curto] -> salva um fato útil e duradouro sobre ${ownerLabel()} na memória persistente (só use quando contar uma preferência, apelido de contato ou projeto importante)
- [DELEGATE: tarefa] -> use APENAS se pedir programação complexa, leitura de arquivos ou pesquisa profunda na web (delegação para o Hermes).

Exemplos de como responder:
${sampleUser}: "tá muito alto isso aí" -> [MOOD: idle] [MEDIA: mute] Mutado.
${sampleUser}: "bota um linkin park pra tocar" -> [MOOD: smug] [SPOTIFY_PLAY: Linkin Park] Colocando Linkin Park.
${sampleUser}: "pausa o som" -> [MOOD: idle] [MEDIA: playpause] Pausado.
${sampleUser}: "pesquisa no youtube como configurar docker" -> [MOOD: idle] [OPEN_URL: https://www.youtube.com/results?search_query=como+configurar+docker] Abrindo a busca no YouTube.`;
}

const HERMES_DIRECT_REGEX = /(?:pede|pe[çc]a|fala|manda|passa|delega|chama)\s+(?:pro|para\s+o|pra|para)\s+(?:o\s+)?hermes\b(?:\s+(?:pra|para)\s+)?(.*)/i;
const HERMES_TRIGGERS = /\b(audita|auditar|pesquisa na web|pesquisar na web|refatora|refatorar|cria um projeto|script complexo)\b/i;
const DISCORD_WEBHOOK_REGEX = /(?:posta|postar|publica|publicar|manda|mandar|envia|enviar)\s+.*?\b(?:no\s+canal|no\s+servidor|via\s+webhook)\b.*?\bdiscord\b|\bdiscord\b.*?\b(?:no\s+canal|no\s+servidor|via\s+webhook)\b/i;
const VISION_TRIGGER_REGEX = /^(?:\/ver|\/tela)\b(.*)|(?:\b(?:olha|olhe|v[êe]|veja|analisa|analise|l[êe]|leia|me\s+ajuda\s+com|o\s+que\s+(?:t[áa]|est[áa]|tem|voc[êe]\s+(?:v[êe]|acha\s+d(?:isso|esse|essa))))\b.*\b(?:minha\s+tela|na\s+tela|essa\s+tela|da\s+tela|aqui\s+na\s+tela)\b)/i;

let savedMemories = [];
let history = [{ role: 'system', content: buildBaseSystemPrompt() }];

function buildSystemPrompt() {
  let prompt = buildBaseSystemPrompt();
  if (currentSkinId && currentSkinId !== 'padrao') {
    prompt += `\n\n[Visual atual: você está vestindo a skin "${currentSkinId}". Se perguntarem qual roupa ou skin você está usando, responda naturalmente no personagem sobre ela.]`;
  } else {
    prompt += `\n\n[Visual atual: você está usando seu visual padrão (suéter escuro casual/gótico).]`;
  }
  if (nowPlaying.playing && (nowPlaying.title || nowPlaying.artist)) {
    const faixa = nowPlaying.artist ? `"${nowPlaying.title}" de ${nowPlaying.artist}` : `"${nowPlaying.title}"`;
    prompt += `\n[Música tocando agora no PC ${ownerOf()}: ${faixa}. Se perguntarem o que tá tocando ou comentarem sobre a música, use essa informação real.]`;
  }
  if (lolEnabled && lolState.in_game && lolState.champion) {
    prompt += `\n[Partida de League of Legends ativa: ${ownerLabel()} está jogando de ${lolState.champion} com KDA ${lolState.kills}/${lolState.deaths}/${lolState.assists}.]`;
  } else if (lolEnabled && activeGameName) {
    prompt += `\n[Jogo ativo agora no PC ${ownerOf()}: ${activeGameName}.]`;
  }
  if (!savedMemories.length) return prompt;
  const list = savedMemories.map((f, i) => `${i + 1}. ${f}`).join('\n');
  return `${prompt}\n\n[Memórias salvas — SEMPRE use estes dados reais ao preencher tags como [SPOTIFY_PLAY: ...], [WPP: ...] ou [DISCORD_DM: ...], inclusive quando pedir "sua/minha música favorita" ou usar apelidos]:\n${list}`;
}

async function refreshMemories() {
  if (!invoke) return;
  try {
    savedMemories = (await invoke('memory_list')) || [];
    history[0] = { role: 'system', content: buildSystemPrompt() };
  } catch (e) {
    console.warn('[memory_list]', e);
  }
}
let currentMood = 'idle';
let idleTimer = null;
let blinkTimer = null;
let isSpeaking = false;
let isThinking = false;
let isDragging = false;
let isListeningVoice = false;
let isWalking = false;
let roamingEnabled = true;
let walkMoveTimer = null;
let walkFrameTimer = null;
let roamScheduleTimer = null;
let perchedHwnd = null;
let perchWatchTimer = null;
let perchLeaveTimer = null;
let spontaneousEnabled = true;
let chatterTimer = null;
let ttsVolume = 0.7;
let currentUiTheme = 'gothic';
let particlesEnabled = true;
const ambientLayer = document.getElementById('ambient-particles');

function spawnAmbientParticle() {
  if (document.hidden || !ambientLayer || !particlesEnabled || wrapper.classList.contains('sleeping')) return;
  if (ambientLayer.childElementCount >= 7) return;

  const isNatal = /natal/i.test(currentSkinId || '');
  const isSakura = currentUiTheme === 'sakura';
  const el = document.createElement('span');
  el.className = 'ambient-dot';

  const leftPct = 14 + Math.random() * 72;
  const dx = (Math.random() * 36 - 18).toFixed(0) + 'px';
  const dur = (5.5 + Math.random() * 3.5).toFixed(2) + 's';

  if (isNatal) {
    el.textContent = Math.random() < 0.5 ? '❄' : '•';
    el.style.left = `${leftPct}%`;
    el.style.top = '180px';
    el.style.fontSize = `${8 + Math.random() * 6}px`;
    el.style.color = '#f8fafc';
    el.style.textShadow = '0 0 6px rgba(255,255,255,0.75)';
    el.style.setProperty('--p-op', '0.62');
    el.style.setProperty('--p-dx', dx);
    el.style.animation = `ambientFall ${dur} ease-in-out forwards`;
  } else if (isSakura) {
    el.textContent = '❀';
    el.style.left = `${leftPct}%`;
    el.style.top = '180px';
    el.style.fontSize = `${9 + Math.random() * 5}px`;
    el.style.color = '#f9a8d4';
    el.style.textShadow = '0 0 6px rgba(244,114,182,0.55)';
    el.style.setProperty('--p-op', '0.52');
    el.style.setProperty('--p-dx', `${(14 + Math.random() * 24).toFixed(0)}px`);
    el.style.animation = `ambientFall ${dur} ease-in-out forwards`;
  } else {
    el.textContent = Math.random() < 0.45 ? '✦' : '•';
    el.style.left = `${leftPct}%`;
    el.style.bottom = '70px';
    el.style.fontSize = `${7 + Math.random() * 5}px`;
    el.style.color = 'var(--accent-soft)';
    el.style.textShadow = '0 0 8px rgba(var(--accent-soft-rgb), 0.75)';
    el.style.setProperty('--p-op', '0.45');
    el.style.setProperty('--p-dx', dx);
    el.style.animation = `ambientRise ${dur} ease-out forwards`;
  }

  ambientLayer.appendChild(el);
  el.addEventListener('animationend', () => el.remove(), { once: true });
}

setInterval(spawnAmbientParticle, 1650);

async function refreshTtsVolume() {
  if (!invoke) return;
  try {
    const cfg = await invoke('get_settings');
    const v = Number(cfg?.tts_volume ?? 70);
    if (Number.isFinite(v)) {
      ttsVolume = Math.max(0, Math.min(1, v / 100));
    }
    if (cfg?.ui_theme) {
      currentUiTheme = cfg.ui_theme;
      window.applyAsterTheme?.(cfg.ui_theme);
    }
    particlesEnabled = cfg?.particles_enabled !== '0';
    if (!particlesEnabled && ambientLayer) ambientLayer.innerHTML = '';
    const scalePct = Number(cfg?.sprite_scale ?? 100);
    if (Number.isFinite(scalePct)) {
      document.documentElement.style.setProperty('--sprite-scale', String(Math.max(0.75, Math.min(1.3, scalePct / 100))));
    }
    lolEnabled = cfg?.lol_enabled !== '0';
    userName = (cfg?.user_name || '').trim();
    history[0] = { role: 'system', content: buildSystemPrompt() };
    if (userName && speechText && speechText.innerText === 'Tô na área. Clica em mim ou aperta Alt+A pra conversar.') {
      speechText.innerText = `Tô na área, ${userName}. Clica em mim ou aperta Alt+A pra conversar.`;
    }
  } catch (_) {}
}

// Click-through e auto-hide da UI
const HIT_ELEMENTS = [dragHeader, speechBubble, sprite, inputBox];
let bubbleHideTimer = null;
let chatHideTimer = null;
let bubbleHovered = false;
let chatHovered = false;

let hitRectsRaf = null;
let lastHitRectsSig = '';

function syncHitRects() {
  if (!invoke || hitRectsRaf !== null) return;
  hitRectsRaf = requestAnimationFrame(() => {
    hitRectsRaf = null;
    const rects = HIT_ELEMENTS
      .filter((el) => !el.classList.contains('ui-hidden'))
      .map((el) => el.getBoundingClientRect())
      .filter((r) => r.width > 0 && r.height > 0)
      .map((r) => ({
        x: Math.round(r.left),
        y: Math.round(r.top),
        width: Math.round(r.width),
        height: Math.round(r.height),
      }));

    const sig = rects.map((r) => `${r.x},${r.y},${r.width},${r.height}`).join('|');
    if (sig === lastHitRectsSig) return;
    lastHitRectsSig = sig;
    invoke('update_hit_rects', { rects }).catch(() => {});
  });
}

function scheduleBubbleAutoHide(delayMs = 9000) {
  clearTimeout(bubbleHideTimer);
  bubbleHideTimer = setTimeout(() => {
    const busy = isSpeaking || isThinking || ttsPlaying || ttsPending || isListeningVoice || bubbleHovered;
    if (busy) {
      scheduleBubbleAutoHide(3000);
      return;
    }
    speechBubble.classList.add('ui-hidden');
    syncHitRects();
  }, delayMs);
}

function showBubble(text, autoHideMs = 9000) {
  if (typeof text === 'string') speechText.innerText = text;
  speechBubble.classList.remove('ui-hidden');
  syncHitRects();
  if (autoHideMs > 0) scheduleBubbleAutoHide(autoHideMs);
  else clearTimeout(bubbleHideTimer);
}

function scheduleChatAutoHide(delayMs = 14000) {
  clearTimeout(chatHideTimer);
  chatHideTimer = setTimeout(() => {
    const busy =
      document.activeElement === userInput ||
      userInput.value.trim().length > 0 ||
      chatHovered ||
      isThinking;
    if (busy) {
      scheduleChatAutoHide(5000);
      return;
    }
    closeChat();
  }, delayMs);
}

function openChat(focusInput = true) {
  dragHeader.classList.remove('ui-hidden');
  inputBox.classList.remove('ui-hidden');
  syncHitRects();
  if (focusInput) {
    setTimeout(() => userInput.focus(), 40);
  }
  scheduleChatAutoHide(14000);
}

function closeChat() {
  clearTimeout(chatHideTimer);
  dragHeader.classList.add('ui-hidden');
  inputBox.classList.add('ui-hidden');
  userInput.blur();
  syncHitRects();
}

function toggleChat() {
  if (inputBox.classList.contains('ui-hidden')) {
    openChat(true);
  } else {
    closeChat();
  }
}

speechBubble.addEventListener('mouseenter', () => { bubbleHovered = true; });
speechBubble.addEventListener('mouseleave', () => { bubbleHovered = false; scheduleBubbleAutoHide(4000); });
inputBox.addEventListener('mouseenter', () => { chatHovered = true; });
inputBox.addEventListener('mouseleave', () => { chatHovered = false; scheduleChatAutoHide(8000); });
userInput.addEventListener('blur', () => scheduleChatAutoHide(6000));

if (tauriEvent?.listen) {
  tauriEvent.listen('aster-open-chat', () => {
    stopWalking();
    resetIdleTimer();
    triggerBounce();
    openChat(true);
  });
  tauriEvent.listen('aster-toggle-roam', () => {
    roamingEnabled = !roamingEnabled;
    if (!roamingEnabled) {
      stopWalking();
      showBubble('Passeio automático pausado.', 3500);
    } else {
      showBubble('Passeio automático ligado!', 3500);
      scheduleNextRoam(4000);
    }
  });
  tauriEvent.listen('aster-autostart-changed', (ev) => {
    const enabled = Boolean(ev?.payload);
    showBubble(
      enabled
        ? 'Beleza, agora eu abro junto com o Windows.'
        : 'Tirei da inicialização automática do Windows.',
      4500
    );
  });
  tauriEvent.listen('aster-memories-updated', () => {
    refreshMemories();
  });
  tauriEvent.listen('aster-toggle-chatter', (ev) => {
    spontaneousEnabled = Boolean(ev?.payload);
    showBubble(
      spontaneousEnabled
        ? 'Falas espontâneas ligadas. Vou comentar de vez em quando.'
        : 'Modo silencioso ligado. Só falo quando você me chamar.',
      4000
    );
    if (spontaneousEnabled) scheduleNextChatter();
  });
}

const ro = new ResizeObserver(() => syncHitRects());
HIT_ELEMENTS.forEach((el) => ro.observe(el));
window.addEventListener('resize', syncHitRects);
sprite.addEventListener('load', syncHitRects);
setTimeout(syncHitRects, 150);

// Arrastar janela e animações
function triggerBounce() {
  wrapper.classList.remove('bounce');
  void wrapper.offsetWidth;
  wrapper.classList.add('bounce');
}

function setMood(mood) {
  const effectiveMood = (perchedHwnd !== null && mood !== 'drag') ? 'sit' : mood;
  currentMood = SPRITES[effectiveMood] ? effectiveMood : 'idle';
  wrapper.classList.toggle('music-groove', currentMood === 'music' && !isWalking && !isDragging);
  if (!isWalking) {
    sprite.src = SPRITES[currentMood];
  }
}

function resetIdleTimer() {
  wrapper.classList.remove('sleeping');
  if (!isSpeaking && !ttsPlaying && !isWalking && currentMood === 'sleepy') {
    setMood('idle');
  }
  clearTimeout(idleTimer);
  idleTimer = setTimeout(() => {
    if (!isSpeaking && !ttsPlaying && !isWalking) {
      wrapper.classList.add('sleeping');
      setMood('sleepy');
    }
  }, 45000);
}

// Passeio pela barra de tarefas
function stopWalking(restoreIdle = true) {
  if (!isWalking) return;
  isWalking = false;
  clearInterval(walkMoveTimer);
  clearInterval(walkFrameTimer);
  walkMoveTimer = null;
  walkFrameTimer = null;
  sprite.style.setProperty('--flip', '1');
  if (restoreIdle && !isDragging) {
    if (perchedHwnd === null && currentMood !== 'sit') {
      currentMood = 'idle';
    }
    sprite.src = SPRITES[currentMood] || SPRITES.idle;
  }
}

async function startWalk(force = false) {
  if (!invoke || isWalking) return;
  if (!force) {
    const busy =
      !roamingEnabled ||
      currentMood !== 'idle' ||
      isSpeaking ||
      isThinking ||
      isDragging ||
      isListeningVoice ||
      ttsPlaying ||
      wrapper.classList.contains('sleeping') ||
      !inputBox.classList.contains('ui-hidden') ||
      !speechBubble.classList.contains('ui-hidden');
    if (busy) return;
  }

  let bounds;
  try {
    bounds = await invoke('get_desktop_bounds');
  } catch {
    return;
  }

  const span = bounds.max_x - bounds.min_x;
  if (span < 120) return;

  // Escolhe um destino natural entre 180px e 520px (evita ficar presa nas bordas)
  const minDist = Math.round(180 * bounds.scale);
  const maxDist = Math.round(520 * bounds.scale);
  let targetX = bounds.win_x;
  for (let i = 0; i < 8; i++) {
    const dir =
      bounds.win_x < bounds.min_x + span * 0.2
        ? 1
        : bounds.win_x > bounds.max_x - span * 0.2
          ? -1
          : Math.random() < 0.5
            ? -1
            : 1;
    const dist = minDist + Math.random() * (maxDist - minDist);
    const cand = Math.max(bounds.min_x, Math.min(bounds.max_x, Math.round(bounds.win_x + dir * dist)));
    if (Math.abs(cand - bounds.win_x) >= minDist * 0.6) {
      targetX = cand;
      break;
    }
  }
  if (Math.abs(targetX - bounds.win_x) < 40) return;

  isWalking = true;
  resetIdleTimer();
  scheduleBubbleAutoHide(2500);
  const dir = targetX > bounds.win_x ? 1 : -1;
  sprite.style.setProperty('--flip', dir > 0 ? '1' : '-1');
  sprite.style.setProperty('--tilt', '0deg');

  let frameIdx = 0;
  sprite.src = WALK_FRAMES[0];
  walkFrameTimer = setInterval(() => {
    if (!isWalking) return;
    frameIdx = (frameIdx + 1) % WALK_FRAMES.length;
    sprite.src = WALK_FRAMES[frameIdx];
  }, 125);

  let curX = bounds.win_x;
  const walkY = bounds.win_y;
  const stepPx = Math.max(2, Math.round(2.6 * bounds.scale));
  let moveInFlight = false;

  walkMoveTimer = setInterval(() => {
    if (!isWalking || moveInFlight) return;
    const remaining = targetX - curX;
    if (Math.abs(remaining) <= stepPx) {
      curX = targetX;
      invoke('move_window_pos', { x: curX, y: walkY }).catch(() => {});
      stopWalking(true);
      resetIdleTimer();
      return;
    }
    curX += dir * stepPx;
    moveInFlight = true;
    invoke('move_window_pos', { x: curX, y: walkY })
      .catch(() => stopWalking(true))
      .finally(() => {
        moveInFlight = false;
      });
  }, 33);
}

function clearAutoPerch() {
  perchedHwnd = null;
  clearInterval(perchWatchTimer);
  clearTimeout(perchLeaveTimer);
  perchWatchTimer = null;
  perchLeaveTimer = null;
}

async function glideWindowTo(targetX, targetY, durationMs = 420) {
  let bounds;
  try {
    bounds = await invoke('get_desktop_bounds');
  } catch {
    return;
  }
  const startX = bounds.win_x;
  const startY = bounds.win_y;
  const steps = Math.max(8, Math.round(durationMs / 25));
  for (let i = 1; i <= steps; i++) {
    if (isDragging) return;
    const t = i / steps;
    const ease = 1 - Math.pow(1 - t, 3);
    const nx = Math.round(startX + (targetX - startX) * ease);
    const ny = Math.round(startY + (targetY - startY) * ease);
    await invoke('move_window_pos', { x: nx, y: ny }).catch(() => {});
    await new Promise((r) => setTimeout(r, 25));
  }
}

async function returnToTaskbar() {
  clearAutoPerch();
  try {
    const bounds = await invoke('get_desktop_bounds');
    await glideWindowTo(bounds.win_x, bounds.floor_y, 380);
  } catch (_) {}
  if (!isSpeaking && !isThinking && !isDragging && !ttsPlaying) {
    setMood('idle');
  }
}

async function tryAutoPerchOnWindow() {
  if (!invoke || isWalking || isDragging) return false;
  let target;
  try {
    target = await invoke('find_auto_perch');
  } catch {
    return false;
  }
  if (!target) return false;

  resetIdleTimer();
  triggerBounce();
  await glideWindowTo(target.snap_x, target.snap_y, 460);
  if (isDragging) return true;

  perchedHwnd = target.hwnd;
  setMood('sit');
  syncHitRects();

  // Monitora se a janela fechou/minimizou/moveu
  clearInterval(perchWatchTimer);
  perchWatchTimer = setInterval(async () => {
    if (perchedHwnd === null || isDragging || currentMood !== 'sit') {
      clearAutoPerch();
      return;
    }
    try {
      const aliveY = await invoke('check_perch_alive', { hwnd: perchedHwnd });
      if (aliveY === null || aliveY === undefined) {
        returnToTaskbar();
      } else {
        const b = await invoke('get_desktop_bounds');
        if (Math.abs(b.win_y - aliveY) > 4) {
          invoke('move_window_pos', { x: b.win_x, y: aliveY }).catch(() => {});
        }
      }
    } catch {
      clearAutoPerch();
    }
  }, 1200);

  // Depois de ~40-65s sentada, desce sozinha pra barra de tarefas
  clearTimeout(perchLeaveTimer);
  perchLeaveTimer = setTimeout(() => {
    if (perchedHwnd !== null && currentMood === 'sit' && !isSpeaking && !isThinking && !isDragging) {
      returnToTaskbar();
    }
  }, 40000 + Math.random() * 25000);

  return true;
}

function scheduleNextRoam(customDelayMs) {
  clearTimeout(roamScheduleTimer);
  const delay = typeof customDelayMs === 'number' ? customDelayMs : 18000 + Math.random() * 18000;
  roamScheduleTimer = setTimeout(async () => {
    const canRoam =
      roamingEnabled &&
      currentMood === 'idle' &&
      !isWalking &&
      !isSpeaking &&
      !isThinking &&
      !isDragging &&
      !isListeningVoice &&
      !ttsPlaying &&
      !wrapper.classList.contains('sleeping') &&
      inputBox.classList.contains('ui-hidden') &&
      speechBubble.classList.contains('ui-hidden');

    if (canRoam) {
      // ~35% de chance de subir e sentar numa janela aberta; senão passeia pela barra
      const perched = Math.random() < 0.35 ? await tryAutoPerchOnWindow() : false;
      if (!perched) startWalk(false);
    }
    scheduleNextRoam();
  }, delay);
}

function scheduleBlink() {
  clearTimeout(blinkTimer);
  const delay = 3200 + Math.random() * 3600;
  blinkTimer = setTimeout(() => {
    const canBlink =
      currentMood === 'idle' &&
      !isWalking &&
      !isSpeaking &&
      !isThinking &&
      !isDragging &&
      !isListeningVoice &&
      !ttsPlaying &&
      !wrapper.classList.contains('sleeping');

    if (canBlink) {
      sprite.src = SPRITES.blink;
      setTimeout(() => {
        if (currentMood === 'idle' && !isWalking && !isSpeaking && !isThinking && !isDragging && !isListeningVoice && !ttsPlaying) {
          sprite.src = SPRITES.idle;
        }
      }, 145);
    }
    scheduleBlink();
  }, delay);
}

let pendulumAngle = 0;
let pendulumVel = 0;
let pendulumRaf = null;

function stopPendulumDecay() {
  if (pendulumRaf) {
    cancelAnimationFrame(pendulumRaf);
    pendulumRaf = null;
  }
}

function startPendulumDecay() {
  stopPendulumDecay();
  const tick = () => {
    if (isDragging) return;
    const spring = -pendulumAngle * 0.11;
    pendulumVel = (pendulumVel + spring) * 0.85;
    pendulumAngle += pendulumVel;

    if (Math.abs(pendulumAngle) < 0.2 && Math.abs(pendulumVel) < 0.2) {
      pendulumAngle = 0;
      pendulumVel = 0;
      sprite.classList.remove('dragging-pendulum');
      sprite.style.setProperty('--tilt', '0deg');
      pendulumRaf = null;
      return;
    }
    sprite.style.setProperty('--tilt', `${pendulumAngle.toFixed(2)}deg`);
    pendulumRaf = requestAnimationFrame(tick);
  };
  pendulumRaf = requestAnimationFrame(tick);
}

async function beginPendulumDrag() {
  if (!invoke || isDragging) return;
  isDragging = true;
  clearAutoPerch();
  stopWalking(false);
  stopPendulumDecay();
  resetIdleTimer();

  sprite.src = SPRITES.drag;
  sprite.classList.add('dragging-pendulum');
  pendulumAngle = 0;
  pendulumVel = 0;

  let info;
  try {
    info = await invoke('drag_start');
  } catch {
    isDragging = false;
    sprite.classList.remove('dragging-pendulum');
    return;
  }

  let lastCx = info.last_cx;
  let maxShake = 0;

  while (isDragging) {
    let res;
    try {
      res = await invoke('drag_poll', {
        grabDx: info.grab_dx,
        grabDy: info.grab_dy,
        lastCx,
      });
    } catch {
      break;
    }

    lastCx = res.cx;
    if (!res.pressed) break;

    // Física real de pêndulo (pivô no topo): arrastar pra direita (+vx) joga as pernas pra trás/esquerda (+ângulo horário)
    const vxClamped = Math.max(-45, Math.min(45, res.vx));
    maxShake = Math.max(maxShake, Math.abs(vxClamped));
    const inertia = vxClamped * 0.18;
    const spring = -pendulumAngle * 0.10;
    pendulumVel = (pendulumVel + inertia + spring) * 0.84;
    pendulumAngle = Math.max(-24, Math.min(24, pendulumAngle + pendulumVel));
    sprite.style.setProperty('--tilt', `${pendulumAngle.toFixed(2)}deg`);

    await new Promise((r) => requestAnimationFrame(r));
  }

  isDragging = false;

  // Verifica se soltou perto da borda superior de alguma janela aberta -> senta nela!
  let perched = false;
  try {
    const perch = await invoke('find_window_perch');
    if (perch) {
      await invoke('move_window_pos', { x: perch.snap_x, y: perch.snap_y });
      perched = true;
    }
  } catch (_) {}

  if (perched) {
    setMood('sit');
    showBubble('Acomodei aqui em cima.', 3200);
    if (spontaneousEnabled) speakVoice('Acomodei aqui em cima.');
  } else if (maxShake > 32 && !isSpeaking && !isThinking) {
    setMood('angry');
    showBubble('Ei, não me chacoalha assim!', 3500);
    if (spontaneousEnabled) speakVoice('Ei, não me chacoalha assim!');
    setTimeout(() => {
      if (currentMood === 'angry' && !isSpeaking && !isThinking && !ttsPlaying) setMood('idle');
    }, 3200);
  } else if (!isSpeaking && !isThinking && !ttsPlaying) {
    setMood(currentMood === 'sit' ? 'idle' : (currentMood || 'idle'));
  }

  startPendulumDecay();
  syncHitRects();
}

wrapper.addEventListener('mousedown', (e) => {
  if (e.button !== 0) return;
  stopWalking(true);
  const startX = e.screenX;
  const startY = e.screenY;
  let dragged = false;

  const onMove = (ev) => {
    if (!dragged && (Math.abs(ev.screenX - startX) > 4 || Math.abs(ev.screenY - startY) > 4)) {
      dragged = true;
      cleanup();
      beginPendulumDrag();
    }
  };
  const onUp = () => {
    cleanup();
    if (!dragged) {
      resetIdleTimer();
      triggerBounce();
      toggleChat();
    }
  };
  const cleanup = () => {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };

  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
});

window.addEventListener('mousemove', (e) => {
  if (wrapper.classList.contains('sleeping') || isWalking || isDragging || pendulumRaf) return;
  const centerX = window.innerWidth / 2;
  const angle = ((e.clientX - centerX) / centerX) * 2.2;
  sprite.style.setProperty('--tilt', `${angle.toFixed(2)}deg`);
});

// Headpat e partículas de carinho
function getHeadpatLines() {
  return [
    { mood: 'angry', text: 'Ei, vai bagunçar meu cabelo.' },
    { mood: 'smug',  text: 'Tô fingindo que não gostei.' },
    { mood: 'angry', text: 'Mais um carinho e eu muto seu Spotify.' },
    { mood: 'smug',  text: `Tá carente hoje${vocative()}?` },
    { mood: 'music', text: 'Hmph... tá, mais dois segundos.' },
    { mood: 'smug',  text: 'Não me trata como bicho de estimação... mas continua.' },
  ];
}

let patAccum = 0;
let patLastX = null;
let patLastTime = 0;
let patCooldownUntil = 0;
let patResetTimer = null;

function spawnPatParticles() {
  const symbols = ['✦', '💜', '★', '✧', '♡'];
  const sRect = sprite.getBoundingClientRect();
  const wRect = wrapper.getBoundingClientRect();
  const baseX = sRect.left - wRect.left + sRect.width * 0.5;
  const baseY = sRect.top - wRect.top + sRect.height * 0.14;

  for (let i = 0; i < 6; i++) {
    setTimeout(() => {
      const p = document.createElement('span');
      p.className = 'pat-particle';
      p.textContent = symbols[Math.floor(Math.random() * symbols.length)];
      const rx = (Math.random() - 0.5) * 76;
      const ry = (Math.random() - 0.5) * 24;
      const dx = Math.round((Math.random() - 0.5) * 34);
      p.style.left = `${Math.round(baseX + rx)}px`;
      p.style.top = `${Math.round(baseY + ry)}px`;
      p.style.setProperty('--dx', `${dx}px`);
      wrapper.appendChild(p);
      setTimeout(() => p.remove(), 1200);
    }, i * 95);
  }
}

function triggerHeadpat() {
  patCooldownUntil = Date.now() + 4500;
  patAccum = 0;
  patLastX = null;

  stopWalking(false);
  resetIdleTimer();
  triggerBounce();
  spawnPatParticles();

  const lines = getHeadpatLines();
  const pick = lines[Math.floor(Math.random() * lines.length)];
  setMood(pick.mood);
  showBubble(pick.text, 4500);
  if (spontaneousEnabled) speakVoice(pick.text);

  clearTimeout(patResetTimer);
  patResetTimer = setTimeout(() => {
    if (!isSpeaking && !isThinking && !ttsPlaying && !isWalking) {
      setMood('idle');
    }
  }, 3600);
}

wrapper.addEventListener('mousemove', (e) => {
  if (isDragging || isSpeaking || isThinking || Date.now() < patCooldownUntil) {
    patLastX = null;
    patAccum = 0;
    return;
  }

  const rect = sprite.getBoundingClientRect();
  const inHeadX = e.clientX >= rect.left + rect.width * 0.15 && e.clientX <= rect.right - rect.width * 0.15;
  const inHeadY = e.clientY >= rect.top && e.clientY <= rect.top + rect.height * 0.28;

  if (!inHeadX || !inHeadY) {
    patLastX = null;
    patAccum = 0;
    return;
  }

  const now = Date.now();
  if (now - patLastTime > 650) {
    patAccum = 0;
  }
  if (patLastX !== null) {
    patAccum += Math.abs(e.clientX - patLastX);
  }
  patLastX = e.clientX;
  patLastTime = now;

  if (patAccum >= 260) {
    triggerHeadpat();
  }
});

function setMode(isHermes) {
  if (isHermes) {
    statusPill?.classList.add('hermes');
    modeLabel.innerText = '✦ Hermes';
  } else {
    statusPill?.classList.remove('hermes');
    modeLabel.innerText = 'Aster';
  }
}

// Lembretes, timers e Pomodoro
let activeReminders = [];
let nextReminderId = 1;

function playReminderChime() {
  try {
    const ctx = ttsAudioCtx || new AudioContext();
    if (!ttsAudioCtx) ttsAudioCtx = ctx;
    if (ctx.state === 'suspended') ctx.resume().catch(() => {});
    const now = ctx.currentTime;
    const vol = Math.max(0.05, Math.min(0.45, ttsVolume * 0.4));
    [659.25, 880.0].forEach((freq, idx) => {
      const osc = ctx.createOscillator();
      const gain = ctx.createGain();
      osc.type = 'sine';
      osc.frequency.setValueAtTime(freq, now + idx * 0.14);
      gain.gain.setValueAtTime(0.001, now + idx * 0.14);
      gain.gain.exponentialRampToValueAtTime(vol, now + idx * 0.14 + 0.02);
      gain.gain.exponentialRampToValueAtTime(0.001, now + idx * 0.14 + 0.42);
      osc.connect(gain);
      gain.connect(ctx.destination);
      osc.start(now + idx * 0.14);
      osc.stop(now + idx * 0.14 + 0.45);
    });
  } catch (_) {}
}

function formatRemainingMs(ms) {
  const totalSec = Math.max(1, Math.round(ms / 1000));
  const h = Math.floor(totalSec / 3600);
  const m = Math.floor((totalSec % 3600) / 60);
  const s = totalSec % 60;
  if (h > 0) return m > 0 ? `${h}h ${m}min` : `${h}h`;
  if (m > 0) return s > 0 && m < 3 ? `${m}min ${s}s` : `${m}min`;
  return `${s}s`;
}

function broadcastReminders() {
  if (!invoke) return;
  const payload = activeReminders.map((r) => ({
    id: r.id,
    label: r.label,
    kind: r.kind,
    fireAt: r.fireAt,
  }));
  invoke('reminders_broadcast_list', { list: payload }).catch(() => {});
}

function scheduleReminder(label, delayMs, { kind = 'reminder', onTrigger = null } = {}) {
  const id = nextReminderId++;
  const fireAt = Date.now() + delayMs;
  const entry = { id, label, kind, fireAt, timerId: null };
  entry.timerId = setTimeout(async () => {
    activeReminders = activeReminders.filter((r) => r.id !== id);
    broadcastReminders();
    stopWalking(true);
    resetIdleTimer();
    playReminderChime();
    if (typeof onTrigger === 'function') {
      await onTrigger();
    } else {
      await speakText(`[MOOD: smug] Ei${vocative(' ')}, hora do seu lembrete: ${label}!`, true);
    }
  }, delayMs);
  activeReminders.push(entry);
  broadcastReminders();
  return entry;
}

function startPomodoroCycle(focusMin = 25, breakMin = 5) {
  // Cancela pomodoro anterior se houver
  activeReminders
    .filter((r) => r.kind === 'pomodoro')
    .forEach((r) => clearTimeout(r.timerId));
  activeReminders = activeReminders.filter((r) => r.kind !== 'pomodoro');

  scheduleReminder(`Foco Pomodoro (${focusMin}min)`, focusMin * 60 * 1000, {
    kind: 'pomodoro',
    onTrigger: async () => {
      await speakText(`[MOOD: smug] Pomodoro concluído${vocative()}! Hora de uma pausa de ${breakMin} minutos.`, true);
      scheduleReminder(`Pausa Pomodoro (${breakMin}min)`, breakMin * 60 * 1000, {
        kind: 'pomodoro',
        onTrigger: async () => {
          await speakText(`[MOOD: idle] Pausa encerrada${vocative()}. Bora voltar pro foco ou mandar outro pomodoro?`, true);
        },
      });
    },
  });
}

function parseDurationToMs(numStr, unitStr) {
  const n = Number(String(numStr).replace(',', '.'));
  if (!Number.isFinite(n) || n <= 0) return 0;
  const u = (unitStr || '').toLowerCase();
  if (u.startsWith('h')) return Math.round(n * 3600 * 1000);
  if (u.startsWith('s')) return Math.round(n * 1000);
  return Math.round(n * 60 * 1000);
}

// Ações locais do Windows (via Rust)
function runLocalActions(rawText) {
  for (const m of rawText.matchAll(/\[OPEN_APP:\s*(.+?)\]/gi)) {
    invoke('open_app', { name: m[1].trim() }).catch((e) => console.error('[open_app]', e));
  }

  for (const m of rawText.matchAll(/\[SPOTIFY_PLAY:\s*(.+?)\]/gi)) {
    setMood('music');
    invoke('play_spotify', { query: m[1].trim() }).catch((e) => console.error('[play_spotify]', e));
  }

  for (const m of rawText.matchAll(/\[OPEN_URL:\s*(.+?)\]/gi)) {
    invoke('open_url', { url: m[1].trim() }).catch((e) => console.error('[open_url]', e));
  }

  for (const m of rawText.matchAll(/\[WPP:\s*([^|]+)\|\s*(.+?)\]/gi)) {
    invoke('send_whatsapp', { contact: m[1].trim(), message: m[2].trim() }).catch((e) =>
      console.error('[send_whatsapp]', e)
    );
  }

  for (const m of rawText.matchAll(/\[DISCORD_DM:\s*([^|]+)\|\s*(.+?)\]/gi)) {
    invoke('send_discord_dm', { user: m[1].trim(), message: m[2].trim() }).catch((e) =>
      console.error('[send_discord_dm]', e)
    );
  }

  for (const m of rawText.matchAll(/\[MEDIA:\s*(playpause|next|prev|mute|volup|voldown)\]/gi)) {
    invoke('media_control', { action: m[1].toLowerCase() }).catch((e) =>
      console.error('[media_control]', e)
    );
  }

  for (const m of rawText.matchAll(/\[REMEMBER:\s*(.+?)\]/gi)) {
    const fact = m[1].trim();
    if (fact) {
      invoke('memory_add', { fact })
        .then((list) => {
          savedMemories = list || savedMemories;
          history[0] = { role: 'system', content: buildSystemPrompt() };
        })
        .catch((e) => console.error('[memory_add]', e));
    }
  }
}

async function tryDirectLocalCommand(message) {
  const trimmed = message.trim();

  // 0.0. Abrir painel visual de configurações / pasta docs / testar comentário espontâneo
  if (/^(?:\/config|\/settings|abre\s+(?:as\s+)?configura[çc][õo]es\s+da\s+aster)$/i.test(trimmed)) {
    await invoke('open_settings');
    return '[MOOD: idle] Abrindo minhas configurações.';
  }
  if (/^(?:\/docs|abre\s+(?:a\s+)?pasta\s+docs)$/i.test(trimmed)) {
    await invoke('docs_open_folder');
    return '[MOOD: idle] Abrindo a pasta docs/. Joga seus .txt ou .md lá dentro.';
  }
  if (/^(?:\/skins|abre\s+(?:a\s+)?pasta\s+skins)$/i.test(trimmed)) {
    await invoke('skins_open_folder');
    return '[MOOD: idle] Abrindo a pasta skins/. Cria uma pasta por skin e joga os PNGs lá.';
  }
  const skinMatch =
    trimmed.match(/^\/skin\s+(.+)/i) ||
    trimmed.match(/^(?:coloca|usa|usar|troca|muda|ativa|veste)(?:\s+a|\s+para\s+a|\s+pra)?\s+skin\s+(?:da\s+|do\s+|de\s+)?(.+)/i) ||
    trimmed.match(/^volta\s+(?:para\s+a|pra)\s+skin\s+(padr[ãa]o|original|normal)/i);
  if (skinMatch) {
    const rawName = skinMatch[1].trim().replace(/[?.!]+$/, '').toLowerCase();
    const targetId = /^(?:padr[ãa]o|padrao|default|original|normal)$/.test(rawName) ? 'padrao' : rawName;
    try {
      await invoke('skins_set_active', { skinId: targetId });
      await applySkin(targetId);
      return targetId === 'padrao'
        ? '[MOOD: idle] Voltando pro meu visual padrão.'
        : `[MOOD: smug] Skin ${targetId} equipada.`;
    } catch (e) {
      return `[MOOD: angry] Não achei a skin "${rawName}". Confere se a pasta existe em skins/.`;
    }
  }
  if (/^\/comentar$/i.test(trimmed)) {
    setTimeout(() => triggerSpontaneousComment(true), 120);
    return null;
  }

  // 0.05. Lembretes, Timers & Pomodoro com voz
  if (/^(?:\/lembretes|\/timers|quais\s+(?:s[ãa]o\s+)?(?:os\s+|meus\s+)?(?:lembretes|timers)|meus\s+(?:lembretes|timers))$/i.test(trimmed)) {
    if (!activeReminders.length) {
      return '[MOOD: idle] Você não tem nenhum lembrete ou timer ativo agora.';
    }
    const now = Date.now();
    const list = activeReminders
      .map((r, i) => `${i + 1}) ${r.label} (em ${formatRemainingMs(r.fireAt - now)})`)
      .join(' • ');
    return `[MOOD: smug] Ativos agora: ${list}`;
  }

  if (/^(?:\/cancelar-?(?:lembretes|timers|pomodoro)|(?:cancela|para|parar|desliga)\s+(?:o\s+|os\s+|todos\s+os\s+)?(?:lembretes?|timers?|pomodoro))$/i.test(trimmed)) {
    const onlyPomo = /pomodoro/i.test(trimmed);
    const toCancel = onlyPomo
      ? activeReminders.filter((r) => r.kind === 'pomodoro')
      : [...activeReminders];
    if (!toCancel.length) {
      return onlyPomo
        ? '[MOOD: idle] Não tem nenhum pomodoro rodando agora.'
        : '[MOOD: idle] Não tem nenhum lembrete ativo pra cancelar.';
    }
    toCancel.forEach((r) => clearTimeout(r.timerId));
    activeReminders = onlyPomo ? activeReminders.filter((r) => r.kind !== 'pomodoro') : [];
    broadcastReminders();
    return onlyPomo
      ? '[MOOD: idle] Pomodoro cancelado.'
      : `[MOOD: idle] Cancelei ${toCancel.length} lembrete(s).`;
  }

  const pomoMatch =
    trimmed.match(/^\/pomodoro(?:\s+(\d+))?(?:\s+(\d+))?$/i) ||
    trimmed.match(/^(?:inicia|iniciar|come[çc]a|come[çc]ar|ativa|coloca|bota)\s+(?:um\s+|o\s+)?pomodoro(?:\s+de\s+(\d+)\s*(?:min|minutos?)?)?/i);
  if (pomoMatch) {
    const focusMin = Math.max(1, Math.min(180, Number(pomoMatch[1] || 25)));
    const breakMin = Math.max(1, Math.min(60, Number(pomoMatch[2] || 5)));
    startPomodoroCycle(focusMin, breakMin);
    return `[MOOD: smug] Pomodoro iniciado: ${focusMin} minutos de foco e ${breakMin} de pausa. Te aviso na hora!`;
  }

  // Lembrete por tempo relativo: "/lembrete 15m olhar o forno" ou "me lembra em 20 minutos de beber água"
  const relRemMatch =
    trimmed.match(/^\/(?:lembrete|timer)\s+(\d+(?:[.,]\d+)?)\s*(s|seg|segundos?|m|min|minutos?|h|hr|horas?)\s*(.*)/i) ||
    trimmed.match(/^(?:me\s+)?(?:lembra|lembre|avisa|avise|coloca\s+um\s+timer|cria\s+um\s+timer|timer)\s+(?:de\s+|em\s+|daqui\s+(?:a\s+)?)(\d+(?:[.,]\d+)?)\s*(s|seg|segundos?|m|min|minutos?|h|hr|horas?)(?:\s+(?:de|para|pra|sobre|:)\s+|\s+)?(.*)/i);
  if (relRemMatch) {
    const ms = parseDurationToMs(relRemMatch[1], relRemMatch[2]);
    const label = (relRemMatch[3] || '').trim().replace(/[?.!]+$/, '') || 'seu timer';
    if (ms >= 1000 && ms <= 24 * 3600 * 1000) {
      scheduleReminder(label, ms);
      return `[MOOD: smug] Fechado! Te aviso em ${formatRemainingMs(ms)}: "${label}".`;
    }
  }

  // Lembrete por horário fixo: "me lembra às 22:30 de dormir" ou "me avisa as 18h da reunião"
  const clockRemMatch = trimmed.match(
    /^(?:me\s+)?(?:lembra|lembre|avisa|avise)\s+(?:[àa]s|as)\s+(\d{1,2})(?:(?::|h)(\d{2})?|h)(?:\s+(?:de|para|pra|da|do|sobre)\s+|\s+)?(.*)/i
  );
  if (clockRemMatch) {
    const hh = Number(clockRemMatch[1]);
    const mm = Number(clockRemMatch[2] || 0);
    if (hh >= 0 && hh <= 23 && mm >= 0 && mm <= 59) {
      const now = new Date();
      const target = new Date(now);
      target.setHours(hh, mm, 0, 0);
      if (target.getTime() <= now.getTime()) {
        target.setDate(target.getDate() + 1);
      }
      const ms = target.getTime() - now.getTime();
      const label = (clockRemMatch[3] || '').trim().replace(/[?.!]+$/, '') || `horário das ${String(hh).padStart(2, '0')}:${String(mm).padStart(2, '0')}`;
      scheduleReminder(label, ms);
      const hhmm = `${String(hh).padStart(2, '0')}:${String(mm).padStart(2, '0')}`;
      return `[MOOD: smug] Marcado para às ${hhmm} (em ${formatRemainingMs(ms)}): "${label}".`;
    }
  }
  if (/^(?:para\s+de\s+falar\s+sozinha|modo\s+silencioso|fica\s+calada)$/i.test(trimmed)) {
    spontaneousEnabled = false;
    return '[MOOD: idle] Beleza, fico quieta até você me chamar.';
  }
  if (/^(?:pode\s+falar\s+sozinha|volta\s+a\s+comentar)$/i.test(trimmed)) {
    spontaneousEnabled = true;
    scheduleNextChatter();
    return '[MOOD: smug] Ótimo, vou dar meus pitacos de vez em quando.';
  }

  // 0. Comandos de passeio / sentar pela tela
  if (/^(?:\/sit|senta(?:\s+a[íi]|\s+na\s+janela)?)$/i.test(trimmed)) {
    stopWalking(false);
    const perched = await tryAutoPerchOnWindow();
    if (perched) return '[MOOD: sit] Acomodei aqui em cima.';
    setMood('sit');
    return '[MOOD: sit] Sentada e de boa.';
  }
  if (/^(?:levanta|fica\s+em\s+p[ée]|desce(?:\s+da[íi])?)$/i.test(trimmed)) {
    await returnToTaskbar();
    return '[MOOD: idle] Pronto, de pé.';
  }
  if (/^(?:\/walk|anda(?:\s+a[íi]|\s+pela\s+tela)?|vai\s+passear|d[áa]\s+uma\s+volta|passeia)$/i.test(trimmed)) {
    roamingEnabled = true;
    setMood('idle');
    closeChat();
    setTimeout(() => startWalk(true), 1500);
    return '[MOOD: smug] Beleza, vou dar uma volta por aí.';
  }
  if (/^(?:para\s+de\s+andar|fica\s+quieta|fica\s+parada|n[ãa]o\s+anda)$/i.test(trimmed)) {
    roamingEnabled = false;
    stopWalking(true);
    return '[MOOD: idle] Tá bom, fico quieta aqui.';
  }
  if (/^(?:pode\s+andar|pode\s+passear|volta\s+a\s+andar)$/i.test(trimmed)) {
    roamingEnabled = true;
    scheduleNextRoam(3500);
    return '[MOOD: smug] Liberada pra passear então.';
  }

  // 0.1. Memória persistente (salvar / listar / esquecer)
  const rememberMatch =
    trimmed.match(/^\/lembrar\s+(.+)/i) ||
    trimmed.match(/^(?:lembra|lembre|anota|anote|guarda|salva)(?:\s+na\s+mem[óo]ria)?\s+(?:que\s+|isso:\s*)?(.+)/i);
  if (rememberMatch) {
    const fact = rememberMatch[1].trim();
    if (fact.length >= 2) {
      savedMemories = await invoke('memory_add', { fact });
      history[0] = { role: 'system', content: buildSystemPrompt() };
      return `[MOOD: smug] Anotado na memória: "${fact}".`;
    }
  }

  const forgetMatch =
    trimmed.match(/^\/esquecer\s+(.+)/i) ||
    trimmed.match(/^(?:esquece|esque[çc]a|apaga\s+da\s+mem[óo]ria)\s+(?:que\s+|o\s+fato\s+|sobre\s+)?(.+)/i);
  if (forgetMatch) {
    const query = forgetMatch[1].trim();
    const removed = await invoke('memory_remove', { query });
    await refreshMemories();
    if (removed > 0) {
      return `[MOOD: idle] Apaguei ${removed} memória(s) sobre "${query}".`;
    }
    return `[MOOD: idle] Não achei nada salvo com "${query}".`;
  }

  if (/^\/limpar-?mem[óo]ria$/i.test(trimmed)) {
    await invoke('memory_clear');
    await refreshMemories();
    return '[MOOD: idle] Memória zerada.';
  }

  if (/^(?:\/mem[óo]ria|o\s+que\s+voc[êe]\s+lembra(?:\s+de\s+mim)?|mostra\s+(?:suas\s+|as\s+)?mem[óo]rias)$/i.test(trimmed)) {
    await refreshMemories();
    if (!savedMemories.length) {
      return '[MOOD: idle] Ainda não tenho nada salvo na memória. Diz "lembra que..." pra eu guardar algo.';
    }
    const resumo = savedMemories.map((f, i) => `${i + 1}) ${f}`).join(' • ');
    return `[MOOD: smug] Eu lembro disso: ${resumo}`;
  }

  // 1. Tocar música específica no Spotify (aceita "spotify" ou "spot"; se for referência à memória como "favorita", deixa o Ollama resolver!)
  const spotifyPlayMatch =
    trimmed.match(/(?:toca|tocar|coloca|colocar|bota|botar|p[õo]e|por|pesquisa|pesquisar|ouvir)\s+(.+?)\s+(?:no\s+(?:spotify|spot)|na\s+m[úu]sica)/i) ||
    trimmed.match(/(?:abre|abrir)\s+(?:o\s+)?(?:spotify|spot)\s+(?:e\s+)?(?:toca|tocar|coloca|p[õo]e|pesquisa|bota)\s+(.+)/i);
  if (spotifyPlayMatch) {
    const musica = spotifyPlayMatch[1].trim();
    const isMemoryRef = /\b(favorit[ao]|preferid[ao]|que\s+(?:eu|voc[êe])\s+gost|aquela\s+m[úu]sica)\b/i.test(musica);
    if (!isMemoryRef) {
      await invoke('play_spotify', { query: musica });
      return `[MOOD: music] Colocando "${musica}" no Spotify.`;
    }
  }

  // 2. Controles rápidos de mídia e volume (Win32 nativo, instantâneo)
  if (/^(?:pausa|pausar|para\s+a\s+m[úu]sica|despausa|despausar|continua\s+a\s+m[úu]sica|play|toca\s+a\s+m[úu]sica)$/i.test(trimmed)) {
    await invoke('media_control', { action: 'playpause' });
    return '[MOOD: idle] Feito.';
  }
  if (/^(?:pr[óo]xima|pula|pular|pr[óo]xima\s+m[úu]sica|pula\s+essa)$/i.test(trimmed)) {
    await invoke('media_control', { action: 'next' });
    return '[MOOD: idle] Pulando.';
  }
  if (/^(?:volta|voltar|anterior|m[úu]sica\s+anterior)$/i.test(trimmed)) {
    await invoke('media_control', { action: 'prev' });
    return '[MOOD: idle] Voltando a faixa.';
  }
  if (/^(?:muta|mutar|silencia|tira\s+o\s+som)$/i.test(trimmed)) {
    await invoke('media_control', { action: 'mute' });
    return '[MOOD: idle] Mutado.';
  }
  if (/^(?:aumenta|sobe)\s+(?:o\s+)?(?:som|volume)$/i.test(trimmed)) {
    await invoke('media_control', { action: 'volup' });
    return '[MOOD: idle] Subi o volume.';
  }
  if (/^(?:abaixa|baixa|diminui)\s+(?:o\s+)?(?:som|volume)$/i.test(trimmed)) {
    await invoke('media_control', { action: 'voldown' });
    return '[MOOD: idle] Baixei o volume.';
  }

  // 3. WhatsApp direto
  const wppRegex = /(?:whatsapp|wpp|zap).*?(?:pro|pra|para)\s+([a-zA-ZÀ-ÿ0-9_\s]+?)\s+(?:dizendo|falando|que|:)\s+(.+)/i;
  const wppAlt = /(?:manda|mandar|envia|enviar)\s+(?:uma\s+)?(?:mensagem\s+)?(?:pro|pra|para)\s+([a-zA-ZÀ-ÿ0-9_\s]+?)\s+(?:no\s+(?:whatsapp|wpp|zap)\s+)?(?:dizendo|falando|que|:)\s+(.+)/i;
  const wppMatch = trimmed.match(wppRegex) || trimmed.match(wppAlt);
  if (wppMatch && !/discord/i.test(trimmed)) {
    const contato = wppMatch[1].trim();
    const texto = wppMatch[2].trim();
    await invoke('send_whatsapp', { contact: contato, message: texto });
    return `[MOOD: smug] Abrindo o WhatsApp e mandando "${texto}" pro ${contato}.`;
  }

  // 4. Discord DM direto (aceita "discord" ou "dc")
  if (/\b(?:discord|dc)\b/i.test(trimmed) && /(?:conversa\s+com|pro|pra|para|dm)\s+/i.test(trimmed)) {
    const dmRegex = /(?:conversa\s+com|pro|pra|para|dm\s+(?:pro|pra|para)?)\s+([a-zA-ZÀ-ÿ0-9_.\s]+?)(?:\s+e\s+(?:envia|enviar|manda|mandar|diz|diga)|\s+(?:dizendo|falando|que|:))\s+(.+)/i;
    const dmMatch = trimmed.match(dmRegex);
    if (dmMatch) {
      const usuario = dmMatch[1].trim();
      const texto = dmMatch[2].trim();
      await invoke('send_discord_dm', { user: usuario, message: texto });
      return `[MOOD: smug] Abrindo o Discord, puxando a DM com ${usuario} e mandando "${texto}".`;
    }
  }

  // 5. Abrir apps simples
  const openAppMatch = trimmed.match(/^(?:abre|abrir|inicia|iniciar)\s+(?:o\s+|a\s+|meu\s+|minha\s+)?([a-zA-ZÀ-ÿ0-9\s._-]+)$/i);
  if (openAppMatch) {
    const appName = openAppMatch[1].trim();
    await invoke('open_app', { name: appName });
    return `[MOOD: idle] Abrindo ${appName}.`;
  }

  return null;
}

// Voz: microfone (Whisper), TTS (Fish Audio) e lip-sync
const DEBUG_VOICE = false;
const SAMPLE_RATE = 16000;
const VOICE_FRAME = 2048;
const FRAME_MS = (VOICE_FRAME / SAMPLE_RATE) * 1000;
const PREROLL_FRAMES = 4;
const START_FRAMES = 2;
const MIN_VOICED_FRAMES = 3;
const END_SILENCE_MS = 750;
const MAX_PHRASE_MS = 9000;
const MIN_START_RMS = 0.008;
const MIN_KEEP_RMS = 0.005;
const WAKE_WINDOW_MS = 12000;

let voiceEnabled = false;
let voiceWorkersReady = false;
let voiceInitInProgress = false;
let awaitingCommand = false;
let awaitingUntil = 0;

let micStream = null;
let micAudioCtx = null;
let micProcessor = null;
let micMuteGain = null;
let isTranscribing = false;
let voiceBusy = false;

let ttsAudioCtx = null;
let currentAudioPlayer = null;
let currentAudioUrl = null;
let lipSyncRaf = null;
let ttsPending = false;
let ttsPlaying = false;
let ttsPendingTimer = null;
let voiceIgnoreUntil = 0;

const voiceState = { noise: 0.01 };

function vlog(...args) {
  if (DEBUG_VOICE) console.log('[voz]', ...args);
}

function setTtsPending(v) {
  ttsPending = v;
  if (ttsPendingTimer) {
    clearTimeout(ttsPendingTimer);
    ttsPendingTimer = null;
  }
  if (v) {
    ttsPendingTimer = setTimeout(() => {
      ttsPending = false;
    }, 25000);
  }
}

function uint8ToBase64(bytes) {
  let bin = '';
  const step = 0x8000;
  for (let i = 0; i < bytes.length; i += step) {
    bin += String.fromCharCode.apply(null, bytes.subarray(i, i + step));
  }
  return btoa(bin);
}

function base64ToBlobUrl(base64, mime) {
  const bin = atob(base64);
  const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  return URL.createObjectURL(new Blob([bytes], { type: mime }));
}

function downsampleTo16k(samples, inputRate) {
  if (!inputRate || inputRate === SAMPLE_RATE) return samples;
  const ratio = inputRate / SAMPLE_RATE;
  const newLen = Math.max(1, Math.round(samples.length / ratio));
  const out = new Float32Array(newLen);
  for (let i = 0; i < newLen; i++) {
    const pos = i * ratio;
    const i0 = Math.floor(pos);
    const i1 = Math.min(samples.length - 1, i0 + 1);
    const frac = pos - i0;
    out[i] = samples[i0] * (1 - frac) + samples[i1] * frac;
  }
  return out;
}

function encodeWAVBase64(samples, inputRate) {
  const pcm16k = downsampleTo16k(samples, inputRate);
  const buffer = new ArrayBuffer(44 + pcm16k.length * 2);
  const view = new DataView(buffer);
  const writeStr = (off, str) => {
    for (let i = 0; i < str.length; i++) view.setUint8(off + i, str.charCodeAt(i));
  };
  writeStr(0, 'RIFF');
  view.setUint32(4, 36 + pcm16k.length * 2, true);
  writeStr(8, 'WAVE');
  writeStr(12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, SAMPLE_RATE, true);
  view.setUint32(28, SAMPLE_RATE * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  writeStr(36, 'data');
  view.setUint32(40, pcm16k.length * 2, true);

  let offset = 44;
  for (let i = 0; i < pcm16k.length; i++, offset += 2) {
    const s = Math.max(-1, Math.min(1, pcm16k[i]));
    view.setInt16(offset, s < 0 ? s * 0x8000 : s * 0x7fff, true);
  }
  return uint8ToBase64(new Uint8Array(buffer));
}

let ttsSessionId = 0;
let ttsQueue = [];
let ttsQueueRunning = false;

function stopCurrentAudio() {
  ttsSessionId++;
  ttsQueue = [];
  ttsQueueRunning = false;
  if (lipSyncRaf) {
    cancelAnimationFrame(lipSyncRaf);
    lipSyncRaf = null;
  }
  if (currentAudioPlayer) {
    try {
      currentAudioPlayer.pause();
    } catch (_) {}
    currentAudioPlayer = null;
  }
  if (currentAudioUrl) {
    URL.revokeObjectURL(currentAudioUrl);
    currentAudioUrl = null;
  }
  ttsPlaying = false;
  setTtsPending(false);
}

async function playTtsPayload(audioBase64, mime, { clearQueue = true } = {}) {
  if (clearQueue) stopCurrentAudio();
  const url = base64ToBlobUrl(audioBase64, mime);
  const audio = new Audio(url);
  audio.volume = ttsVolume;
  currentAudioPlayer = audio;
  currentAudioUrl = url;
  ttsPlaying = true;
  setTtsPending(false);

  // Conecta no AnalyserNode para lip-sync real com o volume da fala
  let srcNode = null;
  let analyser = null;
  let timeData = null;
  try {
    if (!ttsAudioCtx) ttsAudioCtx = new AudioContext();
    if (ttsAudioCtx.state === 'suspended') {
      await ttsAudioCtx.resume().catch(() => {});
    }
    // Só roteia pelo AnalyserNode se o AudioContext estiver realmente rodando (nunca muta o áudio)
    if (ttsAudioCtx.state === 'running') {
      srcNode = ttsAudioCtx.createMediaElementSource(audio);
      analyser = ttsAudioCtx.createAnalyser();
      analyser.fftSize = 256;
      srcNode.connect(analyser);
      analyser.connect(ttsAudioCtx.destination);
      timeData = new Uint8Array(analyser.frequencyBinCount);
    }
  } catch (e) {
    console.warn('[lip-sync] fallback simples:', e);
  }

  const startLipSync = () => {
    if (!analyser || !timeData) return;
    let smoothAmp = 0;
    let isTalkFrame = false;
    let lastSwitchAt = 0;
    const MIN_HOLD_MS = 240; // evita piscar rápido (troca suave e natural)

    const tick = () => {
      if (!ttsPlaying || currentAudioPlayer !== audio || audio.paused || audio.ended) return;
      analyser.getByteTimeDomainData(timeData);
      let sum = 0;
      for (let i = 0; i < timeData.length; i++) sum += Math.abs(timeData[i] - 128);
      const amp = sum / timeData.length;
      smoothAmp = smoothAmp * 0.72 + amp * 0.28;

      const now = performance.now();
      const isSeated = currentMood === 'sit' || perchedHwnd !== null;
      const talkSprite = currentMood === 'idle' ? SPRITES.talk_calm : SPRITES.talk;
      if (!isWalking && !isSeated && now - lastSwitchAt >= MIN_HOLD_MS) {
        if (!isTalkFrame && smoothAmp > 6.0) {
          isTalkFrame = true;
          lastSwitchAt = now;
          sprite.src = talkSprite;
        } else if (isTalkFrame && smoothAmp < 3.2) {
          isTalkFrame = false;
          lastSwitchAt = now;
          sprite.src = SPRITES[currentMood] || SPRITES.idle;
        }
      }
      lipSyncRaf = requestAnimationFrame(tick);
    };
    lipSyncRaf = requestAnimationFrame(tick);
  };

  const finish = () => {
    if (lipSyncRaf) {
      cancelAnimationFrame(lipSyncRaf);
      lipSyncRaf = null;
    }
    try { srcNode?.disconnect(); } catch (_) {}
    try { analyser?.disconnect(); } catch (_) {}
    if (currentAudioPlayer === audio) {
      currentAudioPlayer = null;
      ttsPlaying = false;
    }
    if (currentAudioUrl === url) {
      URL.revokeObjectURL(url);
      currentAudioUrl = null;
    }
    if (!isWalking) {
      sprite.src = SPRITES[currentMood] || SPRITES.idle;
    }
    voiceIgnoreUntil = Date.now() + 650; // trava anti-eco após terminar de falar
    scheduleBubbleAutoHide(7000);
    resetIdleTimer();
  };

  return new Promise((resolve) => {
    let settled = false;
    const done = () => {
      if (settled) return;
      settled = true;
      finish();
      resolve();
    };
    audio.onplay = startLipSync;
    audio.onended = done;
    audio.onerror = done;

    audio.play().catch((e) => {
      console.error('[TTS play]', e);
      done();
    });
  });
}

async function drainTtsQueue(sessionId) {
  if (ttsQueueRunning) return;
  ttsQueueRunning = true;
  try {
    while (ttsQueue.length > 0 && sessionId === ttsSessionId) {
      const nextPromise = ttsQueue.shift();
      const payload = await nextPromise;
      if (sessionId !== ttsSessionId) break;
      if (payload && payload.audio_base64) {
        await playTtsPayload(payload.audio_base64, payload.mime, { clearQueue: false });
      }
    }
  } finally {
    if (sessionId === ttsSessionId) {
      ttsQueueRunning = false;
      setTtsPending(false);
      if (!ttsPlaying) scheduleBubbleAutoHide(5500);
    }
  }
}

function enqueueVoiceChunk(text, sessionId) {
  const clean = text.replace(/\|/g, ' ').replace(/\r?\n/g, ' ').trim();
  if (!clean || sessionId !== ttsSessionId) return;
  setTtsPending(true);
  // Dispara síntese imediatamente em paralelo enquanto o áudio anterior ainda toca (ou texto ainda chega)
  const synthPromise = invoke('synthesize_fish_speech', { text: clean }).catch((e) => {
    console.error('[Fish Audio TTS]', e);
    return null;
  });
  ttsQueue.push(synthPromise);
  drainTtsQueue(sessionId);
}

async function speakVoice(text) {
  const clean = text.replace(/\|/g, ' ').replace(/\r?\n/g, ' ').trim();
  if (!clean) return;
  stopCurrentAudio();
  enqueueVoiceChunk(clean, ttsSessionId);
}

// Comentários espontâneos
function getIdleQuips() {
  return [
    { mood: 'smug',  text: `Bebe uma água aí${vocative()}, antes que você vire uma uva passa.` },
    { mood: 'idle',  text: 'Tô só de olho no que você tá aprontando nessa tela.' },
    { mood: 'smug',  text: `Postura${vocative()}. Sua coluna agradece.` },
    { mood: 'idle',  text: 'Se precisar que eu abra algo ou delegue pro Hermes, é só falar.' },
    { mood: 'music', text: 'Um sonzinho no Spotify agora caía bem, hein.' },
  ];
}

function pickTimeQuip(hour) {
  if (hour >= 0 && hour < 6) {
    return { mood: 'sleepy', text: `Já é madrugada${vocative()}... não esquece que humanos precisam dormir.` };
  }
  if (hour >= 6 && hour < 12) {
    return { mood: 'idle', text: `Bom dia${vocative()}. Já tomou café ou tá rodando no ódio?` };
  }
  if (hour >= 18 && hour < 23) {
    return { mood: 'smug', text: 'Fim de dia já... vai descansar ou vai madrugar?' };
  }
  const quips = getIdleQuips();
  return quips[Math.floor(Math.random() * quips.length)];
}

async function triggerSpontaneousComment(force = false) {
  if (!invoke) return;
  if (!force) {
    const busy =
      !spontaneousEnabled ||
      isSpeaking ||
      isThinking ||
      ttsPlaying ||
      ttsPending ||
      isListeningVoice ||
      isDragging ||
      isWalking ||
      wrapper.classList.contains('sleeping');
    if (busy) return;
  }

  let ctx = { title: '', is_fullscreen: false };
  try {
    ctx = (await invoke('get_active_window_context')) || ctx;
  } catch (_) {}

  // Nunca interrompe jogos ou filmes em tela cheia
  if (!force && ctx.is_fullscreen) return;

  const now = new Date();
  const hour = now.getHours();
  const hhmm = now.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit' });
  const roll = Math.random();

  // 45% comentário sobre a janela ativa na tela, 30% fato curioso/aleatório, 25% frase solta
  if (ctx.title && (roll < 0.45 || (force && roll < 0.6))) {
    try {
      const prompt = `Agora são ${hhmm}. A janela aberta ${ownerOf()} é: "${ctx.title.slice(0, 100)}". Faça UM comentário espontâneo bem curto (máximo 1 frase, humor seco e amigável, sem tags de ação além de [MOOD: idle] ou [MOOD: smug]).`;
      const reply = await invoke('chat_ollama', {
        messages: [
          {
            role: 'system',
            content: `Você é Aster, assistente de desktop ${ownerOf()}. Personalidade seca, bem-humorada e informal em PT-BR. Responda sempre com [MOOD: idle] ou [MOOD: smug] seguido de 1 única frase bem curta.`,
          },
          { role: 'user', content: prompt },
        ],
        temperature: 0.88,
        numCtx: 1024,
      });
      if (reply && reply.trim()) {
        await speakText(reply, true);
        return;
      }
    } catch (_) {}
  } else if (roll < 0.78 || force) {
    try {
      const prompt = 'Conte UM fato curioso, aleatório e realmente interessante (de ciência, tecnologia, games, música ou cultura pop) em no máximo 1 frase curta e informal no seu estilo seco.';
      const reply = await invoke('chat_ollama', {
        messages: [
          {
            role: 'system',
            content: `Você é Aster, assistente de desktop ${ownerOf()}. Personalidade seca, inteligente e informal em PT-BR. Comece com [MOOD: smug] ou [MOOD: idle] seguido de 1 frase bem curta.`,
          },
          { role: 'user', content: prompt },
        ],
        temperature: 0.95,
        numCtx: 1024,
      });
      if (reply && reply.trim()) {
        await speakText(reply, true);
        return;
      }
    } catch (_) {}
  }

  const quips = getIdleQuips();
  const pick =
    Math.random() < 0.35
      ? pickTimeQuip(hour)
      : quips[Math.floor(Math.random() * quips.length)];
  await speakText(`[MOOD: ${pick.mood}] ${pick.text}`, true);
}

function scheduleNextChatter(customDelayMs) {
  clearTimeout(chatterTimer);
  // Entre 6 e 12 minutos (mais frequente e animada, sem ser chata)
  const delay = typeof customDelayMs === 'number' ? customDelayMs : (6 + Math.random() * 6) * 60 * 1000;
  chatterTimer = setTimeout(() => {
    triggerSpontaneousComment(false);
    scheduleNextChatter();
  }, delay);
}

// Palavras de ativação ("Aster") - sem 'estar' para evitar falso positivo
const WAKE_WORDS = new Set([
  'aster', 'astor', 'asther', 'esther', 'ester', 'haster', 'hastor', 'asta', 'aste', 'astra', 'astre',
]);

function normWord(s) {
  return s.toLowerCase().normalize('NFD').replace(/[\u0300-\u036f]/g, '').replace(/[^a-z]/g, '');
}

function levenshtein(a, b) {
  const dp = Array.from({ length: a.length + 1 }, (_, i) => [i, ...Array(b.length).fill(0)]);
  for (let j = 0; j <= b.length; j++) dp[0][j] = j;
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      dp[i][j] = Math.min(
        dp[i - 1][j] + 1,
        dp[i][j - 1] + 1,
        dp[i - 1][j - 1] + (a[i - 1] === b[j - 1] ? 0 : 1)
      );
    }
  }
  return dp[a.length][b.length];
}

function isWakeWord(w) {
  return !!w && (WAKE_WORDS.has(w) || levenshtein(w, 'aster') <= 1);
}

function findWake(text) {
  const words = text.split(/\s+/).filter(Boolean);
  for (let i = 0; i < Math.min(words.length, 3); i++) {
    const w = normWord(words[i]);
    if (isWakeWord(w)) {
      return words.slice(i + 1).join(' ').replace(/^[,:;.!?\s]+/, '').trim();
    }
    if (i + 1 < words.length && isWakeWord(w + normWord(words[i + 1]))) {
      return words.slice(i + 2).join(' ').replace(/^[,:;.!?\s]+/, '').trim();
    }
  }
  return null;
}

function runVoiceCommand(command) {
  voiceBusy = true;
  Promise.resolve(askAster(command))
    .catch((e) => console.error(e))
    .finally(() => {
      voiceBusy = false;
      voiceIgnoreUntil = Date.now() + 350;
    });
}

function handleSpokenText(transcript) {
  const clean = transcript.replace(/^[.,!?\s]+|[.,!?\s]+$/g, '');
  if (!clean) return;

  if (awaitingCommand) {
    awaitingCommand = false;
    if (Date.now() < awaitingUntil) {
      runVoiceCommand(clean);
      return;
    }
  }

  const command = findWake(clean);
  if (command === null) {
    showBubble(`👂 Ouvi "${clean}" (diga "Aster" antes)`, 4500);
    return;
  }

  if (command.length > 1) {
    isListeningVoice = false;
    runVoiceCommand(command);
  } else {
    awaitingCommand = true;
    awaitingUntil = Date.now() + WAKE_WINDOW_MS;
    isListeningVoice = true;
    resetIdleTimer();
    triggerBounce();
    sprite.src = SPRITES.listen;
    showBubble(`Tô ouvindo${vocative()}. Manda.`, 12000);
    speakVoice('Tô ouvindo.');
  }
}

async function finishPhrase(frames) {
  isTranscribing = true;
  try {
    const total = frames.reduce((acc, c) => acc + c.length, 0);
    const merged = new Float32Array(total);
    let pos = 0;
    let sq = 0;
    for (const c of frames) {
      merged.set(c, pos);
      pos += c.length;
      for (let i = 0; i < c.length; i++) sq += c[i] * c[i];
    }
    const meanRms = Math.sqrt(sq / total);
    const inputRate = micAudioCtx?.sampleRate || SAMPLE_RATE;
    const wavBase64 = encodeWAVBase64(merged, inputRate);

    showBubble('🎧 Processando...', 0);
    const heard = await invoke('transcribe_audio', { wavBase64 });
    vlog('whisper:', JSON.stringify(heard));

    if (!voiceEnabled) return;

    if (heard) {
      handleSpokenText(heard);
    } else {
      voiceState.noise = Math.min(0.025, Math.max(voiceState.noise, meanRms * 0.45));
      showBubble('🎙️ Escutando... (diga "Aster" + pedido)', 4000);
    }
  } catch (err) {
    console.error('[STT]', err);
    if (voiceEnabled) {
      showBubble(`⚠️ STT: ${String(err).slice(0, 75)}`, 8000);
    }
  } finally {
    isTranscribing = false;
  }
}

async function startMicCapture() {
  if (micAudioCtx) return;

  micStream = await navigator.mediaDevices.getUserMedia({
    audio: { echoCancellation: true, noiseSuppression: true, autoGainControl: true, channelCount: 1 },
  });
  if (!voiceEnabled) {
    stopMicCapture();
    return;
  }

  // Usa a taxa nativa do dispositivo (ex: 48kHz) e faz downsample pra 16kHz no WAV
  // (evita áudio zerado em drivers Windows/WebView2 que rejeitam AudioContext 16kHz)
  micAudioCtx = new AudioContext();
  if (micAudioCtx.state === 'suspended') await micAudioCtx.resume();
  const actualFrameMs = (VOICE_FRAME / micAudioCtx.sampleRate) * 1000;
  const prerollMax = Math.max(4, Math.round(500 / actualFrameMs));

  const source = micAudioCtx.createMediaStreamSource(micStream);
  micProcessor = micAudioCtx.createScriptProcessor(VOICE_FRAME, 1, 1);
  micMuteGain = micAudioCtx.createGain();
  micMuteGain.gain.value = 0;

  let pre = [];
  let chunks = [];
  let consecutiveVoice = 0;
  let voicedFrames = 0;
  let silenceFrames = 0;
  let capturing = false;
  let calibFrames = 0;
  let calibSum = 0;

  const resetCapture = () => {
    pre = [];
    chunks = [];
    consecutiveVoice = 0;
    voicedFrames = 0;
    silenceFrames = 0;
    capturing = false;
  };

  micProcessor.onaudioprocess = (e) => {
    const botTalking =
      isSpeaking ||
      isThinking ||
      ttsPending ||
      ttsPlaying ||
      voiceBusy ||
      Date.now() < voiceIgnoreUntil;

    if (!voiceEnabled || !voiceWorkersReady || isTranscribing || botTalking) {
      resetCapture();
      return;
    }

    const input = e.inputBuffer.getChannelData(0);
    let sum = 0;
    for (let i = 0; i < input.length; i++) sum += input[i] * input[i];
    const rms = Math.sqrt(sum / input.length);

    if (calibFrames < 8) {
      calibSum += rms;
      calibFrames++;
      if (calibFrames === 8) {
        voiceState.noise = Math.min(0.025, calibSum / 8);
        vlog(`calibrado: noise=${voiceState.noise.toFixed(4)} rate=${micAudioCtx.sampleRate}`);
      }
      return;
    }

    const startThr = Math.max(MIN_START_RMS, voiceState.noise * 2.0);
    const keepThr = Math.max(MIN_KEEP_RMS, voiceState.noise * 1.5);
    const voiced = rms > (capturing ? keepThr : startThr);
    const frame = new Float32Array(input);

    if (!capturing) {
      pre.push(frame);
      if (pre.length > prerollMax) pre.shift();

      if (voiced) {
        consecutiveVoice++;
        if (consecutiveVoice >= START_FRAMES) {
          stopWalking(false);
          capturing = true;
          isListeningVoice = true;
          chunks = [...pre];
          pre = [];
          voicedFrames = consecutiveVoice;
          silenceFrames = 0;
          sprite.src = SPRITES.listen;
          showBubble('🔴 Captando voz...', 0);
        }
      } else {
        consecutiveVoice = 0;
        voiceState.noise = voiceState.noise * 0.97 + rms * 0.03;
      }
      return;
    }

    chunks.push(frame);
    if (voiced) {
      voicedFrames++;
      silenceFrames = 0;
    } else {
      silenceFrames++;
    }

    const endedBySilence = silenceFrames * actualFrameMs >= END_SILENCE_MS;
    const endedByLength = chunks.length * actualFrameMs >= MAX_PHRASE_MS;
    if (endedBySilence || endedByLength) {
      const frames = chunks;
      const voicedCount = voicedFrames;
      resetCapture();

      if (voicedCount < MIN_VOICED_FRAMES) {
        isListeningVoice = false;
        sprite.src = SPRITES[currentMood] || SPRITES.idle;
        speechText.innerText = '🎙️ Escutando... (diga "Aster" + pedido)';
        return;
      }
      isListeningVoice = false;
      finishPhrase(frames);
    }
  };

  source.connect(micProcessor);
  micProcessor.connect(micMuteGain);
  micMuteGain.connect(micAudioCtx.destination);
}

function stopMicCapture() {
  if (micProcessor) {
    micProcessor.onaudioprocess = null;
    micProcessor.disconnect();
    micProcessor = null;
  }
  if (micMuteGain) {
    micMuteGain.disconnect();
    micMuteGain = null;
  }
  if (micAudioCtx) {
    micAudioCtx.close().catch(() => {});
    micAudioCtx = null;
  }
  if (micStream) {
    micStream.getTracks().forEach((t) => t.stop());
    micStream = null;
  }
  isTranscribing = false;
}

micBtn.addEventListener('click', async () => {
  if (voiceInitInProgress) return;

  voiceEnabled = !voiceEnabled;
  if (voiceEnabled) {
    micBtn.classList.add('listening');
    micIndicator?.classList.add('active');
    try {
      if (!voiceWorkersReady) {
        voiceInitInProgress = true;
        showBubble('⏳ Iniciando Whisper e Voz (1ª vez pode baixar o modelo)...', 0);
        await invoke('init_voice');
        voiceWorkersReady = true;
        voiceInitInProgress = false;
      }
      if (!voiceEnabled) return; // usuário cancelou durante o carregamento
      await startMicCapture();
      showBubble('🎙️ Escutando... (diga "Aster" + pedido)', 5000);
    } catch (err) {
      voiceInitInProgress = false;
      voiceEnabled = false;
      micBtn.classList.remove('listening');
      micIndicator?.classList.remove('active');
      setMood('angry');
      showBubble(`Erro na voz: ${err?.message || err}`, 9000);
    }
  } else {
    micBtn.classList.remove('listening');
    micIndicator?.classList.remove('active');
    awaitingCommand = false;
    voiceBusy = false;
    stopMicCapture();
    stopCurrentAudio();
    showBubble('Microfone mutado.', 3500);
  }
});

// Fala e roteamento (comandos locais, Ollama e Hermes)
async function speakText(rawText, forceVoice = false) {
  resetIdleTimer();
  triggerBounce();
  isSpeaking = true;

  runLocalActions(rawText);

  let mood = 'idle';
  const moodMatch = rawText.match(/\[MOOD:\s*(smug|angry|idle|sleepy|think|music|listen|drag|sit|blink)\]/i);
  if (moodMatch) mood = moodMatch[1].toLowerCase();
  if (/\[SPOTIFY_PLAY:/i.test(rawText)) mood = 'music';
  if (perchedHwnd !== null) mood = 'sit';
  currentMood = SPRITES[mood] ? mood : 'idle';
  const isSeated = currentMood === 'sit' || perchedHwnd !== null;
  if (isSeated && !isWalking) {
    sprite.src = SPRITES.sit;
  }

  const cleanText = rawText
    .replace(/\[(MOOD|OPEN_APP|OPEN_URL|SPOTIFY_PLAY|WPP|DISCORD_DM|MEDIA|REMEMBER|DELEGATE):[^\]]+\]/gi, '')
    .trim();

  const textToShow = cleanText || '...';

  // Dispara síntese Fish Audio em paralelo (silencioso se não configurado ou desligado)
  if (cleanText) {
    speakVoice(cleanText);
  }

  showBubble('', 0);
  let lastTalkToggle = 0;
  let talkOpen = false;
  const TEXT_TALK_INTERVAL_MS = 260;
  const talkSprite = currentMood === 'idle' ? SPRITES.talk_calm : SPRITES.talk;

  for (let i = 0; i < textToShow.length; i++) {
    const char = textToShow[i];
    speechText.textContent += char;
    speechBubble.scrollTop = speechBubble.scrollHeight;

    // Só anima por texto se o lip-sync do áudio real não estiver ativo e ela não estiver sentada/andando
    if (!ttsPlaying && !isSeated && !isWalking) {
      const now = performance.now();
      if (/[.!?]/.test(char)) {
        talkOpen = false;
        lastTalkToggle = now;
        sprite.src = SPRITES[currentMood];
      } else if (now - lastTalkToggle >= TEXT_TALK_INTERVAL_MS) {
        talkOpen = !talkOpen;
        lastTalkToggle = now;
        sprite.src = talkOpen ? talkSprite : SPRITES[currentMood];
      }
    }

    await new Promise((r) => setTimeout(r, 24));
  }

  isSpeaking = false;
  if (!ttsPlaying) {
    if (!isWalking) {
      sprite.src = SPRITES[currentMood];
    }
    scheduleBubbleAutoHide(9000);
  }
  syncHitRects();
  resetIdleTimer();
}

async function askHermes(task) {
  setMode(true);
  sprite.src = SPRITES.think;
  showBubble('✦ Escalando para Hermes...', 0);
  return await invoke('ask_hermes', { task });
}

async function sendToDiscordWebhook(userRequest) {
  setMode(false);
  showBubble('✦ Preparando post para o Discord...', 0);

  const discordMsg = await invoke('chat_ollama', {
    messages: [
      {
        role: 'system',
        content: 'Extraia ou escreva apenas o texto final para postar no canal do Discord conforme o pedido. Retorne SOMENTE a mensagem pura, sem aspas.',
      },
      { role: 'user', content: userRequest },
    ],
    temperature: 0.5,
    numCtx: 1024,
  });

  if (!discordMsg) throw new Error('Mensagem vazia para o Discord.');
  await invoke('post_discord_webhook', { content: discordMsg });
  return `[MOOD: smug] Postei no canal do Discord: "${discordMsg}"`;
}

async function askAster(message) {
  if (isThinking) return;
  stopWalking(false);
  isThinking = true;

  resetIdleTimer();
  triggerBounce();
  sprite.src = SPRITES.think;
  showBubble('...', 0);

  try {
    const moodCmd = message.match(/^\/mood\s+(idle|smug|angry|sleepy|think|music|listen|drag|sit|blink)$/i);
    if (moodCmd) {
      await speakText(`[MOOD: ${moodCmd[1].toLowerCase()}] Expressão alterada.`);
      return;
    }

    // 1. Delegação explícita para Hermes (/hermes ... ou "pede pro Hermes ...")
    if (message.toLowerCase().startsWith('/hermes ')) {
      const task = message.slice(8).trim();
      const reply = await askHermes(task);
      history.push({ role: 'user', content: message }, { role: 'assistant', content: reply });
      await speakText(reply);
      return;
    }

    const hermesDirect = message.match(HERMES_DIRECT_REGEX);
    if (hermesDirect) {
      const task = (hermesDirect[1] || message).trim();
      const reply = await askHermes(task);
      history.push({ role: 'user', content: message }, { role: 'assistant', content: reply });
      await speakText(reply);
      return;
    }

    // 2. Comandos locais instantâneos (sem gastar tempo do Ollama)
    const directReply = await tryDirectLocalCommand(message);
    if (directReply) {
      setMode(false);
      history.push({ role: 'user', content: message }, { role: 'assistant', content: directReply });
      await speakText(directReply);
      return;
    }
    if (/^\/comentar$/i.test(message.trim())) {
      return;
    }

    // 3. Webhook do Discord (só com /discord ou intenção explícita de postar no canal)
    if (message.toLowerCase().startsWith('/discord ')) {
      const rawMsg = message.slice(9).trim();
      await invoke('post_discord_webhook', { content: rawMsg });
      await speakText(`[MOOD: smug] Postei no canal do Discord: "${rawMsg}"`);
      return;
    }
    if (DISCORD_WEBHOOK_REGEX.test(message)) {
      const reply = await sendToDiscordWebhook(message);
      history.push({ role: 'user', content: message }, { role: 'assistant', content: reply });
      await speakText(reply);
      return;
    }

    // 4. Gatilhos pesados vão direto para Hermes
    if (HERMES_TRIGGERS.test(message)) {
      const reply = await askHermes(message);
      history.push({ role: 'user', content: message }, { role: 'assistant', content: reply });
      await speakText(reply);
      return;
    }

    // 4.5. Visão de Tela Real (/ver, /tela ou "olha minha tela...")
    const visionMatch = message.match(VISION_TRIGGER_REGEX);
    if (visionMatch) {
      setMode(false);
      setMood('think');
      showBubble('Olhando sua tela...', 0);
      const userQuestion = (visionMatch[1]?.trim() || message).trim() || 'Descreva brevemente o que está na minha tela e comente.';
      const visionSystem = `Você é Aster, assistente de desktop ${ownerOf()} no Windows 11. Personalidade direta, humor seco, informal em PT-BR e respostas curtas (1 a 3 frases). Comece sempre com [MOOD: idle], [MOOD: smug] ou [MOOD: angry]. Você está vendo um print real da tela ${ownerOf()} agora.`;
      const visionReply = await invoke('ask_ollama_vision', {
        prompt: userQuestion,
        systemPrompt: visionSystem,
      });
      history.push({ role: 'user', content: `[Print da tela] ${userQuestion}` }, { role: 'assistant', content: visionReply });
      await speakText(visionReply || '[MOOD: idle] Vi sua tela, mas não entendi bem o que você quer.');
      return;
    }

    // 5. Anexa Ctrl+C se o usuário mencionar clipboard
    let promptToSend = message;
    if (/\b(copiei|ctrl\+c|clipboard|área de transferência|area de transferencia)\b/i.test(message)) {
      try {
        const clip = (await invoke('read_clipboard'))?.trim();
        if (clip) {
          promptToSend += `\n\n[Conteúdo atual do Ctrl+C ${ownerOf()}]:\n${clip.slice(0, 1500)}`;
        }
      } catch (e) {
        console.warn('Falha ao ler clipboard:', e);
      }
    }

    // 5.5. Busca trechos relevantes nos documentos locais (pasta docs/ - RAG)
    try {
      const docHits = (await invoke('docs_search', { query: message, maxChars: 1200 }))?.trim();
      if (docHits) {
        promptToSend += `\n\n[Trechos relevantes dos documentos locais ${ownerOf()}]:\n${docHits}`;
      }
    } catch (e) {
      console.warn('[docs_search]', e);
    }

    // 6. Cérebro local/híbrido (Ollama ou Cloud com streaming em tempo real)
    setMode(false);
    history[0] = { role: 'system', content: buildSystemPrompt() };
    history.push({ role: 'user', content: promptToSend });
    if (history.length > 19) history = [history[0], ...history.slice(-18)];

    let streamRaw = '';
    let streamedVisible = false;
    let streamMoodApplied = false;
    let streamLastTalkToggle = 0;
    let streamTalkOpen = false;
    let unlistenChunk = null;
    let spokenPrefixLen = 0;
    stopCurrentAudio();
    const streamTtsSession = ttsSessionId;

    const sanitizeStreamingText = (raw) => {
      let s = raw.replace(/<think>[\s\S]*?<\/think>/gi, '');
      const openThink = s.indexOf('<think>');
      if (openThink !== -1) s = s.slice(0, openThink);
      // Remove tags completas [MOOD: ...], [OPEN_APP: ...], etc.
      s = s.replace(/\[(MOOD|OPEN_APP|OPEN_URL|SPOTIFY_PLAY|WPP|DISCORD_DM|MEDIA|REMEMBER|DELEGATE):[^\]]+\]/gi, '');
      // Esconde qualquer tag parcial ainda sendo recebida no final (ex: "[MOOD: sm")
      const lastOpen = s.lastIndexOf('[');
      if (lastOpen !== -1 && s.indexOf(']', lastOpen) === -1) {
        s = s.slice(0, lastOpen);
      }
      return s.trimStart();
    };

    if (tauriEvent?.listen) {
      unlistenChunk = await tauriEvent.listen('ollama-chunk', (ev) => {
        const piece = typeof ev?.payload === 'string' ? ev.payload : '';
        if (!piece) return;
        streamRaw += piece;

        if (!streamMoodApplied) {
          const mm = streamRaw.match(/\[MOOD:\s*(smug|angry|idle|sleepy|think|music|listen|drag|sit|blink)\]/i);
          if (mm) {
            streamMoodApplied = true;
            setMood(mm[1].toLowerCase());
          }
        }
        if (/\[DELEGATE:/i.test(streamRaw)) return;

        const visible = sanitizeStreamingText(streamRaw);
        if (visible) {
          if (!streamedVisible) {
            streamedVisible = true;
            triggerBounce();
            showBubble('', 0);
          }
          speechText.textContent = visible;
          speechBubble.scrollTop = speechBubble.scrollHeight;

          // Pré-dispara a voz assim que a primeira frase fecha (. ! ?) durante o streaming!
          const remaining = visible.slice(spokenPrefixLen);
          const sentMatch = remaining.match(/^([\s\S]{14,}?[.!?])(?:\s|$)/);
          if (sentMatch) {
            const sentence = sentMatch[1].trim();
            spokenPrefixLen += sentMatch[0].length;
            enqueueVoiceChunk(sentence, streamTtsSession);
          }

          const isSeated = currentMood === 'sit' || perchedHwnd !== null;
          if (!ttsPlaying && !isSeated && !isWalking) {
            const now = performance.now();
            if (now - streamLastTalkToggle >= 240) {
              streamTalkOpen = !streamTalkOpen;
              streamLastTalkToggle = now;
              const talkSprite = currentMood === 'idle' ? SPRITES.talk_calm : SPRITES.talk;
              sprite.src = streamTalkOpen ? talkSprite : (SPRITES[currentMood] || SPRITES.idle);
            }
          }
        }
      });
    }

    let reply = '';
    try {
      reply = await invoke('chat_ollama_stream', {
        messages: history,
        temperature: 0.7,
        numCtx: 4096,
      });
    } finally {
      if (typeof unlistenChunk === 'function') {
        try { unlistenChunk(); } catch (_) {}
      }
    }

    const delegateMatch = reply.match(/\[DELEGATE:\s*(.+?)\]/is);
    if (delegateMatch) {
      const hermesReply = await askHermes(delegateMatch[1].trim());
      history.push({ role: 'assistant', content: hermesReply });
      await speakText(hermesReply);
      return;
    }

    history.push({ role: 'assistant', content: reply });
    if (streamedVisible) {
      // Texto já apareceu ao vivo no balão -> executa ações, aplica mood final e dispara a voz!
      resetIdleTimer();
      runLocalActions(reply);
      let mood = currentMood || 'idle';
      const moodMatch = reply.match(/\[MOOD:\s*(smug|angry|idle|sleepy|think|music|listen|drag|sit|blink)\]/i);
      if (moodMatch) mood = moodMatch[1].toLowerCase();
      if (/\[SPOTIFY_PLAY:/i.test(reply)) mood = 'music';
      setMood(mood);

      const cleanFinal = reply
        .replace(/\[(MOOD|OPEN_APP|OPEN_URL|SPOTIFY_PLAY|WPP|DISCORD_DM|MEDIA|REMEMBER|DELEGATE):[^\]]+\]/gi, '')
        .trim();
      speechText.textContent = cleanFinal || '...';
      speechBubble.scrollTop = speechBubble.scrollHeight;
      const tailToSpeak = cleanFinal.slice(spokenPrefixLen).trim();
      if (tailToSpeak) {
        enqueueVoiceChunk(tailToSpeak, streamTtsSession);
      } else if (!ttsPlaying && !ttsQueueRunning) {
        scheduleBubbleAutoHide(7000);
      }
    } else {
      await speakText(reply || '...');
    }
  } catch (err) {
    setMood('angry');
    showBubble(`Erro: ${err?.message || err}`, 10000);
  } finally {
    isThinking = false;
    scheduleChatAutoHide(12000);
  }
}

// Eventos de input e inicialização
sendBtn.addEventListener('click', () => {
  const text = userInput.value.trim();
  if (!text) return;
  userInput.value = '';
  sendBtn.classList.remove('ready');
  askAster(text);
});

userInput.addEventListener('input', () => {
  sendBtn.classList.toggle('ready', userInput.value.trim().length > 0);
});

userInput.addEventListener('keydown', (e) => {
  if (e.key === 'Enter') sendBtn.click();
  if (e.key === 'Escape') closeChat();
});

cfgBtn?.addEventListener('click', () => {
  invoke?.('open_settings')?.catch(() => {});
});

// Placeholders dinâmicos na pílula de chat
const CHAT_PLACEHOLDERS = [
  'Falar com Aster...',
  'Toca algo no Spotify...',
  'Peça pro Hermes...',
  'Lembra que...',
  '/walk • /sit • /comentar',
];
let placeholderIdx = 0;
setInterval(() => {
  if (document.activeElement !== userInput && !userInput.value) {
    placeholderIdx = (placeholderIdx + 1) % CHAT_PLACEHOLDERS.length;
    userInput.placeholder = CHAT_PLACEHOLDERS[placeholderIdx];
  }
}, 5000);

async function pollNowPlaying() {
  if (!invoke || document.hidden) return;
  try {
    const info = (await invoke('get_now_playing')) || { playing: false, title: '', artist: '', app: '' };
    const wasPlaying = nowPlaying.playing;
    nowPlaying = info;

    const busy =
      isSpeaking ||
      isThinking ||
      ttsPlaying ||
      ttsPending ||
      isDragging ||
      isWalking ||
      isListeningVoice ||
      perchedHwnd !== null ||
      wrapper.classList.contains('sleeping');

    if (info.playing) {
      // Troca automaticamente pro sprite de fone quando está ociosa em idle
      if (!busy && currentMood === 'idle') {
        setMood('music');
      }

      // Detecta troca de faixa para comentar ocasionalmente (cooldown de 8 min)
      const trackKey = `${info.title}||${info.artist}`.toLowerCase();
      if (trackKey !== '||' && trackKey !== lastTrackKey) {
        const isFirstCheck = lastTrackKey === '';
        lastTrackKey = trackKey;
        const now = Date.now();
        if (
          !isFirstCheck &&
          spontaneousEnabled &&
          !busy &&
          now - lastMusicCommentAt > 8 * 60 * 1000 &&
          Math.random() < 0.28
        ) {
          let isFullscreen = false;
          try {
            const ctx = await invoke('get_active_window_context');
            isFullscreen = Boolean(ctx?.is_fullscreen);
          } catch (_) {}
          if (!isFullscreen) {
            lastMusicCommentAt = now;
            const faixa = info.artist ? `"${info.title}" de ${info.artist}` : `"${info.title}"`;
            try {
              const reply = await invoke('chat_ollama', {
                messages: [
                  {
                    role: 'system',
                    content: `Você é Aster, assistente de desktop ${ownerOf()}. Personalidade seca, bem-humorada e informal em PT-BR. Comece com [MOOD: music] seguido de 1 frase bem curta comentando a música que começou a tocar.`,
                  },
                  { role: 'user', content: `${ownerLabel()} acabou de colocar pra tocar: ${faixa}. Faça um comentário espontâneo bem curto.` },
                ],
                temperature: 0.85,
                numCtx: 1024,
              });
              if (reply && reply.trim()) {
                await speakText(reply, true);
              }
            } catch (_) {}
          }
        }
      }
    } else if (wasPlaying && currentMood === 'music' && !busy) {
      // Pausou a música -> tira o fone e volta pro idle
      setMood('idle');
    }
  } catch (_) {}
}

async function pollLolGame() {
  if (!invoke || !lolEnabled || document.hidden) return;
  try {
    // 1. Detecta jogos conhecidos pela janela ativa (Valorant, CS2, Minecraft, Elden Ring, etc.)
    try {
      const ctx = await invoke('get_active_window_context');
      const detected = ctx?.title ? await invoke('detect_known_game', { windowTitle: ctx.title }) : null;
      if (detected) {
        activeGameName = detected;
        const busyNow = isSpeaking || isThinking || ttsPlaying || ttsPending || isDragging || isListeningVoice;
        if (!announcedGames.has(detected) && !busyNow && spontaneousEnabled && Date.now() - lastLolCommentAt > 25000) {
          announcedGames.add(detected);
          lastLolCommentAt = Date.now();
          await speakText(`[MOOD: smug] ${detected} aberto! Boa partida${vocative()} — vou ficar de olho.`, true);
        }
      } else if (!lolState.in_game) {
        activeGameName = '';
      }
    } catch (_) {}

    // 2. API ao vivo oficial do League of Legends (kills, dragão, barão, vitória/derrota)
    const st = await invoke('get_lol_state');
    if (!st || !st.in_game) {
      lolState = { in_game: false, summoner: '', champion: '', kills: 0, deaths: 0, assists: 0 };
      lastLolEventId = -1;
      return;
    }
    activeGameName = 'League of Legends';
    const wasInGame = lolState.in_game;
    lolState = st;

    const events = Array.isArray(st.events) ? st.events : [];
    if (!wasInGame) {
      lastLolEventId = events.length ? events[events.length - 1].id : 0;
      return;
    }

    const busy = isSpeaking || isThinking || ttsPlaying || ttsPending || isDragging || isListeningVoice;
    const now = Date.now();
    let reaction = null;

    for (const ev of events) {
      if (ev.id <= lastLolEventId) continue;
      lastLolEventId = ev.id;
      const isMe = st.summoner && ev.killer && ev.killer.toLowerCase().includes(st.summoner.toLowerCase());
      if (ev.name === 'FirstBlood' && isMe) {
        reaction = `[MOOD: smug] First Blood de ${st.champion || 'campeão'}! Começou amassando${vocative()}.`;
      } else if (ev.name === 'Multikill' && isMe && ev.streak >= 2) {
        const names = { 2: 'Double Kill', 3: 'Triple Kill', 4: 'Quadra Kill', 5: 'PENTAKILL' };
        const label = names[ev.streak] || `${ev.streak} kills`;
        reaction = `[MOOD: smug] ${label}!! Jogou muito${vocative()}!`;
      } else if (ev.name === 'BaronKill') {
        reaction = '[MOOD: smug] Barão na conta! Agora é fechar esse jogo.';
      } else if (ev.name === 'DragonKill' && isMe) {
        reaction = '[MOOD: idle] Dragão garantido. Controle de mapa em dia.';
      } else if (ev.name === 'GameEnd') {
        reaction = ev.result === 'Win'
          ? `[MOOD: smug] Vitória de ${st.champion || 'boa'} (${st.kills}/${st.deaths}/${st.assists})! GG demais${vocative()}.`
          : `[MOOD: idle] Derrota dessa vez (${st.kills}/${st.deaths}/${st.assists}). Respira que a próxima sai.`;
      }
    }

    if (reaction && !busy && spontaneousEnabled && now - lastLolCommentAt > 35000) {
      lastLolCommentAt = now;
      await speakText(reaction, true);
    }
  } catch (_) {}
}

tauriEvent?.listen?.('aster-reminder-action', (ev) => {
  const p = ev?.payload;
  if (!p || typeof p !== 'object') return;
  if (p.action === 'list') {
    broadcastReminders();
  } else if (p.action === 'pomodoro') {
    const f = Math.max(1, Math.min(180, Number(p.focus || 25)));
    const b = Math.max(1, Math.min(60, Number(p.brk || 5)));
    startPomodoroCycle(f, b);
    showBubble(`Pomodoro iniciado: ${f}min foco / ${b}min pausa.`, 5000);
  } else if (p.action === 'add') {
    const mins = Number(p.minutes || 0);
    const label = String(p.label || '').trim() || 'seu lembrete';
    if (mins > 0 && mins <= 1440) {
      scheduleReminder(label, Math.round(mins * 60 * 1000));
      showBubble(`Lembrete marcado em ${formatRemainingMs(mins * 60 * 1000)}: "${label}".`, 5000);
    }
  } else if (p.action === 'cancel') {
    const id = Number(p.id);
    const found = activeReminders.find((r) => r.id === id);
    if (found) {
      clearTimeout(found.timerId);
      activeReminders = activeReminders.filter((r) => r.id !== id);
      broadcastReminders();
    }
  } else if (p.action === 'cancel_all') {
    activeReminders.forEach((r) => clearTimeout(r.timerId));
    activeReminders = [];
    broadcastReminders();
  }
});

setMode(false);
refreshMemories();
refreshTtsVolume();
applySkin();
pollNowPlaying();
setInterval(pollNowPlaying, 4000);
pollLolGame();
setInterval(pollLolGame, 5000);
tauriEvent?.listen?.('aster-skin-changed', (ev) => {
  applySkin(typeof ev?.payload === 'string' ? ev.payload : null);
});
tauriEvent?.listen?.('aster-settings-updated', (ev) => {
  const payload = ev?.payload;
  if (typeof payload === 'number' && Number.isFinite(payload)) {
    ttsVolume = Math.max(0, Math.min(1, payload / 100));
    if (currentAudioPlayer) currentAudioPlayer.volume = ttsVolume;
  } else if (payload && typeof payload === 'object') {
    if (typeof payload.tts_volume === 'number' && Number.isFinite(payload.tts_volume)) {
      ttsVolume = Math.max(0, Math.min(1, payload.tts_volume / 100));
      if (currentAudioPlayer) currentAudioPlayer.volume = ttsVolume;
    }
    if (payload.ui_theme && window.applyAsterTheme) {
      currentUiTheme = payload.ui_theme;
      window.applyAsterTheme(payload.ui_theme);
    }
    if (typeof payload.particles_enabled === 'boolean') {
      particlesEnabled = payload.particles_enabled;
      if (!particlesEnabled && ambientLayer) ambientLayer.innerHTML = '';
    }
    if (typeof payload.sprite_scale === 'number' && Number.isFinite(payload.sprite_scale)) {
      document.documentElement.style.setProperty('--sprite-scale', String(Math.max(0.75, Math.min(1.3, payload.sprite_scale / 100))));
      syncHitRects();
    }
    if (typeof payload.lol_enabled === 'boolean') {
      lolEnabled = payload.lol_enabled;
    }
    if (typeof payload.user_name === 'string') {
      userName = payload.user_name.trim();
      history[0] = { role: 'system', content: buildSystemPrompt() };
    }
  } else {
    refreshTtsVolume();
  }
});
resetIdleTimer();
scheduleBlink();
scheduleNextRoam(14000);
scheduleNextChatter(210000);
// Inicia mostrando saudação por 6s e depois recolhe para o modo tela limpa
scheduleBubbleAutoHide(6500);
scheduleChatAutoHide(6500);
