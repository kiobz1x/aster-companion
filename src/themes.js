// Presets de cores compartilhados entre a janela principal (index.html) e Configurações (settings.html)
window.ASTER_THEMES = {
  gothic: {
    name: 'Glass Gothic',
    desc: 'Roxo profundo e lavanda — elegante e sombrio',
    accent: '#8b5cf6',
    accentBright: '#a78bfa',
    accentSoft: '#c4b5fd',
    accentText: '#ede9fe',
    accentDeep: '#5b21b6',
    accentMid: '#7c3aed',
    accentRgb: '139, 92, 246',
    accentSoftRgb: '167, 139, 250',
    bgGlow: '#1d1236',
    bgMid: '#0b0914',
    bgDeep: '#07060d',
    surface: 'rgba(22, 17, 36, 0.76)',
    sidebar: 'rgba(15, 11, 26, 0.88)',
    bubbleBg: 'rgba(13, 11, 20, 0.92)',
    pillBg: 'rgba(15, 12, 24, 0.88)',
  },
  kinkou: {
    name: 'Kinkou Emerald',
    desc: 'Verde jade e esmeralda — contraste vibrante',
    accent: '#10b981',
    accentBright: '#34d399',
    accentSoft: '#6ee7b7',
    accentText: '#ecfdf5',
    accentDeep: '#065f46',
    accentMid: '#059669',
    accentRgb: '16, 185, 129',
    accentSoftRgb: '52, 211, 153',
    bgGlow: '#0a2920',
    bgMid: '#06120f',
    bgDeep: '#040a08',
    surface: 'rgba(11, 28, 22, 0.78)',
    sidebar: 'rgba(7, 19, 15, 0.9)',
    bubbleBg: 'rgba(8, 19, 15, 0.93)',
    pillBg: 'rgba(9, 22, 17, 0.9)',
  },
  crimson: {
    name: 'Crimson Natal',
    desc: 'Vermelho vinho e rubi — quente e marcante',
    accent: '#e11d48',
    accentBright: '#fb7185',
    accentSoft: '#fda4af',
    accentText: '#fff1f2',
    accentDeep: '#881337',
    accentMid: '#be123c',
    accentRgb: '225, 29, 72',
    accentSoftRgb: '251, 113, 133',
    bgGlow: '#320d18',
    bgMid: '#13070b',
    bgDeep: '#0a0406',
    surface: 'rgba(34, 13, 20, 0.78)',
    sidebar: 'rgba(22, 9, 13, 0.9)',
    bubbleBg: 'rgba(22, 9, 14, 0.93)',
    pillBg: 'rgba(26, 11, 16, 0.9)',
  },
  cyber: {
    name: 'Cyber Ice',
    desc: 'Ciano elétrico e azul noturno — visual futurista',
    accent: '#06b6d4',
    accentBright: '#22d3ee',
    accentSoft: '#67e8f9',
    accentText: '#ecfeff',
    accentDeep: '#155e75',
    accentMid: '#0891b2',
    accentRgb: '6, 182, 212',
    accentSoftRgb: '34, 211, 238',
    bgGlow: '#0b2533',
    bgMid: '#061018',
    bgDeep: '#04090e',
    surface: 'rgba(11, 26, 36, 0.78)',
    sidebar: 'rgba(8, 18, 26, 0.9)',
    bubbleBg: 'rgba(8, 18, 26, 0.93)',
    pillBg: 'rgba(9, 22, 31, 0.9)',
  },
  sakura: {
    name: 'Sakura Nocturne',
    desc: 'Rosa suave e ameixa escura — leve e estilizado',
    accent: '#ec4899',
    accentBright: '#f472b6',
    accentSoft: '#f9a8d4',
    accentText: '#fdf2f8',
    accentDeep: '#9d174d',
    accentMid: '#db2777',
    accentRgb: '236, 72, 153',
    accentSoftRgb: '244, 114, 182',
    bgGlow: '#311126',
    bgMid: '#130811',
    bgDeep: '#0a0409',
    surface: 'rgba(33, 14, 27, 0.78)',
    sidebar: 'rgba(22, 9, 18, 0.9)',
    bubbleBg: 'rgba(22, 9, 18, 0.93)',
    pillBg: 'rgba(25, 11, 21, 0.9)',
  },
  obsidian: {
    name: 'Obsidian Mono',
    desc: 'Preto fosco e prata — minimalismo puro',
    accent: '#a1a1aa',
    accentBright: '#d4d4d8',
    accentSoft: '#e4e4e7',
    accentText: '#fafafa',
    accentDeep: '#3f3f46',
    accentMid: '#52525b',
    accentRgb: '161, 161, 170',
    accentSoftRgb: '212, 212, 216',
    bgGlow: '#1f1f23',
    bgMid: '#0c0c0e',
    bgDeep: '#070709',
    surface: 'rgba(24, 24, 27, 0.82)',
    sidebar: 'rgba(15, 15, 18, 0.92)',
    bubbleBg: 'rgba(14, 14, 17, 0.94)',
    pillBg: 'rgba(18, 18, 21, 0.92)',
  },
};

window.applyAsterTheme = function (themeId) {
  const t = window.ASTER_THEMES[themeId] || window.ASTER_THEMES.gothic;
  const r = document.documentElement.style;
  r.setProperty('--accent', t.accent);
  r.setProperty('--accent-bright', t.accentBright);
  r.setProperty('--accent-soft', t.accentSoft);
  r.setProperty('--accent-text', t.accentText);
  r.setProperty('--accent-deep', t.accentDeep);
  r.setProperty('--accent-mid', t.accentMid);
  r.setProperty('--accent-rgb', t.accentRgb);
  r.setProperty('--accent-soft-rgb', t.accentSoftRgb);
  r.setProperty('--bg-glow', t.bgGlow);
  r.setProperty('--bg-mid', t.bgMid);
  r.setProperty('--bg-deep', t.bgDeep);
  r.setProperty('--surface', t.surface);
  r.setProperty('--sidebar', t.sidebar);
  r.setProperty('--bubble-bg', t.bubbleBg);
  r.setProperty('--pill-bg', t.pillBg);
  return t;
};
