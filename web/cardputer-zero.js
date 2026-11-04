// Clickable Cardputer Zero keyboard for HarbourOS's existing Emscripten SDL2
// build. The flat chassis is supplied by cardputer-zero.html.
// The browser's physical keyboard continues to go directly to SDL.
(() => {
  'use strict';

  // Appearance belongs to the simulator chassis, never to HarbourOS itself.
  // Start in dark mode, and remember the user's chosen enclosure finish.
  const appearanceSwitch = document.getElementById('theme-toggle');
  const appearanceKey = 'harbour-cardputer-simulator-theme';

  function setAppearance(mode) {
    const dark = mode === 'dark';
    document.body.dataset.simulatorTheme = dark ? 'dark' : 'light';
    appearanceSwitch.textContent = dark ? 'Light mode' : 'Dark mode';
    appearanceSwitch.setAttribute('aria-label',
      dark ? 'Switch to light mode' : 'Switch to dark mode');
    appearanceSwitch.setAttribute('aria-pressed', String(dark));
  }

  let initialMode = 'dark';
  try {
    const stored = localStorage.getItem(appearanceKey);
    if (stored === 'light' || stored === 'dark') initialMode = stored;
  } catch (_) {
    // Private browsing or storage restrictions should not break simulator input.
  }
  setAppearance(initialMode);
  appearanceSwitch.addEventListener('click', () => {
    const next = document.body.dataset.simulatorTheme === 'dark' ? 'light' : 'dark';
    setAppearance(next);
    try {
      localStorage.setItem(appearanceKey, next);
    } catch (_) {}
  });

  const rows = [
    ['ESC','1','2','3','4','5','6','7','8','9','0'],
    ['TAB','Q','W','E','R','T','Y','U','I','O','P'],
    ['↑','A','S','D','F','G','H','J','K','L','↵','DEL'],
    ['FN','CTRL','Z','X','C','V','B','N','M',',','.','SPACE']
  ];

  const secondary = {
    '1':'!', '2':'@', '3':'#', '4':'$', '5':'%',
    '6':'^', '7':'&', '8':'*', '9':'(', '0':')',
    'Q':'~', 'W':'-', 'E':'+', 'R':'=', 'T':'/',
    'Y':'\\', 'U':'[', 'I':']', 'O':'{', 'P':'}'
  };
  const specialKeys = new Set(['ESC','TAB','DEL','FN','CTRL']);
  const fnArrow = { 'F':'ArrowUp', 'Z':'ArrowLeft', 'X':'ArrowDown', 'C':'ArrowRight' };
  const keyCodes = {
    Escape:27, Tab:9, Backspace:8, Enter:13, Space:32,
    Control:17, ArrowUp:38, ArrowLeft:37, ArrowDown:40, ArrowRight:39,
    Comma:188, Period:190
  };
  const keyNames = {
    ESC: ['Escape','Escape'],
    TAB: ['Tab','Tab'],
    DEL: ['Backspace','Backspace'],
    '↑': ['ArrowUp','ArrowUp'],
    '↵': ['Enter','Enter'],
    SPACE: ['Space',' '],
    CTRL: ['ControlLeft','Control']
  };

  const keyboard = document.getElementById('keyboard');
  let fnLatched = false;
  let ctrlLatched = false;
  let modifierFn = null;
  let modifierCtrl = null;
  const buttonsByLabel = new Map();

  function definition(label) {
    if (label in keyNames) {
      const pair = keyNames[label];
      return { code:pair[0], key:pair[1],
        keyCode:keyCodes[pair[1]] || 0, char:pair[1] === ' ' ? ' ' : null };
    }
    if (label.startsWith('Arrow')) {
      return { code:label, key:label, keyCode:keyCodes[label], char:null };
    }
    if (label.length === 1 && /[A-Z]/.test(label)) {
      return { code:'Key' + label, key:label.toLowerCase(),
        keyCode:label.charCodeAt(0), char:label.toLowerCase() };
    }
    if (label.length === 1 && /[0-9]/.test(label)) {
      return { code:'Digit' + label, key:label,
        keyCode:label.charCodeAt(0), char:label };
    }
    if (label === ',') {
      return { code:'Comma', key:',', keyCode:188, char:',' };
    }
    if (label === '.') {
      return { code:'Period', key:'.', keyCode:190, char:'.' };
    }
    return null;
  }

  // Emscripten SDL2 listens for DOM keyboard events. Match the existing 3D
  // simulator's keycode, which and charCode forwarding conventions.
  function fire(type, def) {
    const charCode = type === 'keypress' && def.char ?
      def.char.codePointAt(0) : 0;
    const keyCode = type === 'keypress' ? charCode : def.keyCode;
    const ev = new KeyboardEvent(type, {
      key:def.key, code:def.code, bubbles:true, cancelable:true,
      ctrlKey:ctrlLatched
    });
    Object.defineProperty(ev, 'keyCode', {get:() => keyCode});
    Object.defineProperty(ev, 'which', {get:() => keyCode});
    Object.defineProperty(ev, 'charCode', {get:() => charCode});
    window.dispatchEvent(ev);
  }

  function setModVisual() {
    if (modifierFn) modifierFn.classList.toggle('sticky', fnLatched);
    if (modifierCtrl) modifierCtrl.classList.toggle('sticky', ctrlLatched);
  }

  function start(label) {
    if (label === 'FN') {
      fnLatched = !fnLatched;
      setModVisual();
      return null;
    }
    if (label === 'CTRL') {
      ctrlLatched = !ctrlLatched;
      // Send actual Ctrl modifier key transitions to SDL.
      fire(ctrlLatched ? 'keydown' : 'keyup', definition('CTRL'));
      setModVisual();
      return null;
    }
    // Cardputer Zero Fn navigation: F=Up, Z=Left, X=Down, C=Right.
    const mapped = fnLatched && fnArrow[label] ? fnArrow[label] : label;
    const def = definition(mapped);
    if (!def) return null;
    fire('keydown', def);
    if (def.char && !ctrlLatched) fire('keypress', def);
    return def;
  }

  function finish(def) {
    if (!def) return;
    fire('keyup', def);
    if (fnLatched) fnLatched = false;
    if (ctrlLatched) {
      fire('keyup', definition('CTRL'));
      ctrlLatched = false;
    }
    setModVisual();
  }

  function bindPress(element, label) {
    let held = null;
    const down = (e) => {
      if (e.button !== undefined && e.button !== 0) return;
      e.preventDefault();
      element.setPointerCapture?.(e.pointerId);
      if (held) return;
      element.classList.add('pressed');
      held = {def:start(label)};
    };
    const up = (e) => {
      if (!held) return;
      e.preventDefault();
      finish(held.def);
      held = null;
      element.classList.remove('pressed');
    };
    element.addEventListener('pointerdown', down);
    element.addEventListener('pointerup', up);
    element.addEventListener('pointercancel', up);
    element.addEventListener('lostpointercapture', up);
  }

  rows.forEach(keys => {
    const row = document.createElement('div');
    row.className = 'key-row';
    keys.forEach(label => {
      const key = document.createElement('button');
      key.className = 'key';
      key.type = 'button';
      key.setAttribute('aria-label', label);
      key.title = label;

      const legend = document.createElement('div');
      legend.className = 'legend';
      if (specialKeys.has(label)) legend.classList.add('special');
      const main = document.createElement('span');
      main.textContent = label;
      legend.appendChild(main);

      if (secondary[label]) {
        const alt = document.createElement('span');
        alt.className = 'secondary';
        alt.textContent = secondary[label];
        legend.appendChild(alt);
      }

      const cap = document.createElement('div');
      cap.className = 'keycap';
      key.appendChild(legend);
      key.appendChild(cap);
      row.appendChild(key);
      buttonsByLabel.set(label, key);
      bindPress(key, label);
    });
    keyboard.appendChild(row);
  });

  modifierFn = buttonsByLabel.get('FN');
  modifierCtrl = buttonsByLabel.get('CTRL');
  document.querySelectorAll('.nav-button').forEach(button => {
    bindPress(button, button.dataset.key);
  });
  document.getElementById('restart').addEventListener('click', () => {
    window.location.reload();
  });

  // Prevent browser shortcuts from stealing common HarbourOS commands.
  window.addEventListener('keydown', e => {
    if (!e.isTrusted) return;
    if (e.ctrlKey && ['s','b','q'].includes(e.key.toLowerCase())) {
      e.preventDefault();
    }
    const label = e.code.startsWith('Key') ? e.code.slice(3) :
      e.key === 'Enter' ? '↵' : e.key === 'Backspace' ? 'DEL' :
      e.key === 'Escape' ? 'ESC' : null;
    buttonsByLabel.get(label)?.classList.add('pressed');
  });
  window.addEventListener('keyup', e => {
    if (!e.isTrusted) return;
    const label = e.code.startsWith('Key') ? e.code.slice(3) :
      e.key === 'Enter' ? '↵' : e.key === 'Backspace' ? 'DEL' :
      e.key === 'Escape' ? 'ESC' : null;
    buttonsByLabel.get(label)?.classList.remove('pressed');
  });
})();
