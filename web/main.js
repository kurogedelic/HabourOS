// Harbour OS Web — Three.js 白い一体型ハンドヘルド + QWERTY キーボード。
// SDL (Emscripten) が #canvas に描いた画面を筐体スクリーンのテクスチャにし、
// 3D キー / スクリーン上のポインタ操作を合成 DOM イベントとして SDL に渡す。

import * as THREE from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { RoomEnvironment } from 'three/addons/environments/RoomEnvironment.js';

const AMBER = '#ffb040';

const stage = document.getElementById('stage');
const sdlCanvas = document.getElementById('canvas');
const [OS_W, OS_H] = window.Module.osSize;

// ---------------------------------------------------------------- renderer

const renderer = new THREE.WebGLRenderer({ canvas: stage, antialias: true });
renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
renderer.toneMapping = THREE.ACESFilmicToneMapping;
renderer.toneMappingExposure = 0.95;
renderer.shadowMap.enabled = true;
renderer.shadowMap.type = THREE.PCFSoftShadowMap;

const BG = '#d9d7d2';
const scene = new THREE.Scene();
scene.background = new THREE.Color(BG);
const pmrem = new THREE.PMREMGenerator(renderer);
scene.environment = pmrem.fromScene(new RoomEnvironment(), 0.04).texture;
scene.environmentIntensity = 0.55;

const camera = new THREE.PerspectiveCamera(26, 4 / 3, 1, 400);

scene.add(new THREE.HemisphereLight('#ffffff', '#b9b5ad', 0.9));
const key = new THREE.DirectionalLight('#fffaf2', 2.0);
key.position.set(-16, 42, 20);
key.castShadow = true;
key.shadow.mapSize.set(2048, 2048);
Object.assign(key.shadow.camera, { left: -30, right: 30, top: 30, bottom: -30, near: 1, far: 120 });
key.shadow.bias = -0.0004;
key.shadow.radius = 6;
scene.add(key);

// 机：影だけを落とす
const floor = new THREE.Mesh(
  new THREE.PlaneGeometry(400, 400),
  new THREE.ShadowMaterial({ color: '#4a463f', opacity: 0.22 })
);
floor.rotation.x = -Math.PI / 2;
floor.receiveShadow = true;
scene.add(floor);

// ---------------------------------------------------------------- enclosure
// 白い一体型ハンドヘルド。上面奥にスクリーン、手前にキーボード。
// 側面と画面両脇に水平グルーブを刻む。

const body = new THREE.Group();
scene.add(body);

const shellMat = new THREE.MeshStandardMaterial({ color: '#efeee9', roughness: 0.42 });
const grooveMat = new THREE.MeshStandardMaterial({ color: '#c9c7c0', roughness: 0.7 });
const glassMat = new THREE.MeshPhysicalMaterial({ color: '#0e0e0f', roughness: 0.18, clearcoat: 1, clearcoatRoughness: 0.08 });

const PITCH = 1.7, CAP = 1.5, CAP_H = 0.55, TRAVEL = 0.22;
const KB_U = 15, KB_ROWS = 5;

// 横長ディスプレイ：キーボード幅いっぱいに取り、高さは OS の縦横比から決める
const screenAspect = OS_W / OS_H;
const BEZEL = 0.8;
const SCREEN_W = KB_U * PITCH - BEZEL * 2;
const SCREEN_H = Math.min(SCREEN_W / screenAspect, 16);

const BODY_W = KB_U * PITCH + 3.2;
const BODY_H = 2.2;
const REAR = 3.2, GAP = 1.8, FRONT = 1.6;
const BODY_D = REAR + SCREEN_H + BEZEL * 2 + GAP + KB_ROWS * PITCH + FRONT;
const TOP = BODY_H;
const RADIUS = 0.1;

const shell = new THREE.Mesh(new RoundedBoxGeometry(BODY_W, BODY_H, BODY_D, 3, RADIUS), shellMat);
shell.position.y = BODY_H / 2;
shell.castShadow = shell.receiveShadow = true;
body.add(shell);

const zRear = -BODY_D / 2;
const screenZ = zRear + REAR + BEZEL + SCREEN_H / 2;
const kbZ0 = zRear + REAR + SCREEN_H + BEZEL * 2 + GAP + PITCH / 2;

function groove(len, axis, pos) {
  const g = new THREE.Mesh(
    new THREE.BoxGeometry(axis === 'x' ? len : 0.1, 0.1, axis === 'z' ? len : 0.1),
    grooveMat
  );
  g.position.copy(pos);
  body.add(g);
}

// 側面を一周する 3 本のグルーブ
for (const y of [0.55, 0.95, 1.35]) {
  const lx = BODY_W - RADIUS * 2, lz = BODY_D - RADIUS * 2;
  groove(lx, 'x', new THREE.Vector3(0, y, BODY_D / 2 - 0.03));
  groove(lx, 'x', new THREE.Vector3(0, y, -BODY_D / 2 + 0.03));
  groove(lz, 'z', new THREE.Vector3(BODY_W / 2 - 0.03, y, 0));
  groove(lz, 'z', new THREE.Vector3(-BODY_W / 2 + 0.03, y, 0));
}

// 画面奥のスピーカーグルーブ
for (let i = 0; i < 4; i++) {
  groove(SCREEN_W + BEZEL * 2, 'x', new THREE.Vector3(0, TOP - 0.02, zRear + 0.9 + i * 0.5));
}

// スクリーン：黒ガラスのパネルを上面にはめ込む
const panel = new THREE.Mesh(
  new RoundedBoxGeometry(SCREEN_W + BEZEL * 2, 0.2, SCREEN_H + BEZEL * 2, 2, 0.015),
  glassMat
);
panel.position.set(0, TOP - 0.04, screenZ);
panel.receiveShadow = true;
body.add(panel);

const screenTex = new THREE.CanvasTexture(sdlCanvas);
screenTex.colorSpace = THREE.SRGBColorSpace;
screenTex.magFilter = THREE.NearestFilter;
screenTex.minFilter = THREE.LinearFilter;
screenTex.generateMipmaps = false;
const texSize = [sdlCanvas.width, sdlCanvas.height];

const screenMat = new THREE.ShaderMaterial({
  uniforms: { map: { value: screenTex }, res: { value: new THREE.Vector2(OS_W, OS_H) } },
  vertexShader: /* glsl */`
    varying vec2 vUv;
    void main() { vUv = uv; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }
  `,
  fragmentShader: /* glsl */`
    uniform sampler2D map;
    uniform vec2 res;
    varying vec2 vUv;
    void main() {
      vec3 c = texture2D(map, vUv).rgb;
      // 走査線と周辺減光で控えめな CRT 感
      float scan = 0.9 + 0.1 * sin(vUv.y * res.y * 3.14159265 * 2.0);
      vec2 d = vUv - 0.5;
      float vig = 1.0 - dot(d, d) * 0.7;
      c = c * scan * vig + vec3(0.012, 0.008, 0.004);
      gl_FragColor = vec4(c, 1.0);
      #include <colorspace_fragment>
    }
  `,
});
const screen = new THREE.Mesh(new THREE.PlaneGeometry(SCREEN_W, SCREEN_H), screenMat);
screen.rotation.x = -Math.PI / 2;
screen.position.set(0, TOP + 0.07, screenZ);
body.add(screen);

// 画面のガラス反射
const glass = new THREE.Mesh(
  new THREE.PlaneGeometry(SCREEN_W + BEZEL * 2 - 0.2, SCREEN_H + BEZEL * 2 - 0.2),
  new THREE.MeshPhysicalMaterial({
    color: '#000000', roughness: 0.06, transparent: true, opacity: 0.12, envMapIntensity: 1.4,
  })
);
glass.rotation.x = -Math.PI / 2;
glass.position.set(0, TOP + 0.08, screenZ);
body.add(glass);

function labelTexture(text, { w = 512, h = 64, color = '#8a877f', font = '500 30px "Helvetica Neue", Helvetica, Arial, sans-serif', align = 'left' } = {}) {
  const c = document.createElement('canvas');
  c.width = w; c.height = h;
  const g = c.getContext('2d');
  g.fillStyle = color;
  g.font = font;
  g.textBaseline = 'middle';
  g.textAlign = align;
  g.fillText(text, align === 'center' ? w / 2 : 4, h / 2);
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  t.anisotropy = 8;
  return t;
}

// 画面とキーボードの間：名前と LED
const infoZ = zRear + REAR + SCREEN_H + BEZEL * 2 + GAP / 2;
const logo = new THREE.Mesh(
  new THREE.PlaneGeometry(6, 0.75),
  new THREE.MeshBasicMaterial({ map: labelTexture('harbour', { font: '500 40px "Helvetica Neue", Helvetica, Arial, sans-serif' }), transparent: true })
);
logo.rotation.x = -Math.PI / 2;
logo.position.set(-SCREEN_W / 2 - BEZEL + 3, TOP + 0.01, infoZ);
body.add(logo);

const led = new THREE.Mesh(new THREE.CircleGeometry(0.16, 20), new THREE.MeshBasicMaterial({ color: AMBER }));
led.rotation.x = -Math.PI / 2;
led.position.set(SCREEN_W / 2 + BEZEL - 0.2, TOP + 0.01, infoZ);
body.add(led);
const ledGlow = new THREE.PointLight(AMBER, 0.4, 3);
ledGlow.position.set(led.position.x, TOP + 0.5, infoZ);
body.add(ledGlow);

// ---------------------------------------------------------------- keyboard

// [code, label, widthU, keyCode, char, shiftChar]
const L = (c) => [`Key${c}`, c, 1, c.charCodeAt(0), c.toLowerCase(), c];
const D = (n, s) => [`Digit${n}`, n, 1, 48 + Number(n), n, s];
const ROWS = [
  [
    ['Escape', 'esc', 1, 27],
    D('1', '!'), D('2', '@'), D('3', '#'), D('4', '$'), D('5', '%'),
    D('6', '^'), D('7', '&'), D('8', '*'), D('9', '('), D('0', ')'),
    ['Minus', '-', 1, 189, '-', '_'], ['Equal', '=', 1, 187, '=', '+'],
    ['Backspace', 'delete', 2, 8],
  ],
  [
    ['Tab', 'tab', 1.5, 9],
    ...'QWERTYUIOP'.split('').map(L),
    ['BracketLeft', '[', 1, 219, '[', '{'], ['BracketRight', ']', 1, 221, ']', '}'],
    ['Backslash', '\\', 1.5, 220, '\\', '|'],
  ],
  [
    ['ControlLeft', 'control', 1.75, 17],
    ...'ASDFGHJKL'.split('').map(L),
    ['Semicolon', ';', 1, 186, ';', ':'], ['Quote', "'", 1, 222, "'", '"'],
    ['Enter', 'return', 2.25, 13],
  ],
  [
    ['ShiftLeft', 'shift', 2, 16],
    ...'ZXCVBNM'.split('').map(L),
    ['Comma', ',', 1, 188, ',', '<'], ['Period', '.', 1, 190, '.', '>'], ['Slash', '/', 1, 191, '/', '?'],
    ['ShiftRight', 'shift', 1, 16],
    ['ArrowUp', '↑', 1, 38],
    ['Delete', 'del', 1, 46],
  ],
  [
    ['Home', 'home', 1.25, 36], ['End', 'end', 1.25, 35],
    ['PageUp', 'pg up', 1.25, 33], ['PageDown', 'pg dn', 1.25, 34],
    ['Space', '', 7, 32, ' ', ' '],
    ['ArrowLeft', '←', 1, 37], ['ArrowDown', '↓', 1, 40], ['ArrowRight', '→', 1, 39],
  ],
];
const KEY_NAMES = {
  Escape: 'Escape', Backspace: 'Backspace', Tab: 'Tab', Enter: 'Enter',
  ControlLeft: 'Control', ShiftLeft: 'Shift', ShiftRight: 'Shift',
  ArrowUp: 'ArrowUp', ArrowDown: 'ArrowDown', ArrowLeft: 'ArrowLeft', ArrowRight: 'ArrowRight',
  Delete: 'Delete', Home: 'Home', End: 'End', PageUp: 'PageUp', PageDown: 'PageDown',
};
const MODIFIERS = new Set(['ShiftLeft', 'ShiftRight', 'ControlLeft']);

// キー印字：コンデンス系サンセリフのイタリック。
// 単語キーは左下に小文字、記号キーは上段にシフト記号、文字キーは中央に 1 文字。
const LEGEND_FONT = '"Roboto Condensed", "Helvetica Neue", Arial, sans-serif';
const LEGEND_COLOR = '#55524c';
await document.fonts.load(`italic 400 40px ${LEGEND_FONT}`).catch(() => {});

function keyLegend(def, aspect) {
  const h = 128, w = Math.round(h * aspect);
  const c = document.createElement('canvas');
  c.width = w; c.height = h;
  const g = c.getContext('2d');
  g.fillStyle = LEGEND_COLOR;
  g.textBaseline = 'alphabetic';
  const { label, char, shiftChar } = def;
  const twoTier = char !== undefined && shiftChar !== undefined && shiftChar !== char.toUpperCase();
  if (label.length > 1) {
    g.font = `italic 400 34px ${LEGEND_FONT}`;
    g.textAlign = 'left';
    g.fillText(label, 16, h - 22);
  } else if (twoTier) {
    g.font = `italic 400 36px ${LEGEND_FONT}`;
    g.textAlign = 'center';
    g.fillText(shiftChar, w / 2, h / 2 - 8);
    g.fillText(char, w / 2, h / 2 + 34);
  } else {
    g.font = `italic 400 44px ${LEGEND_FONT}`;
    g.textAlign = 'center';
    g.fillText(label, w / 2, h / 2 + 16);
  }
  const t = new THREE.CanvasTexture(c);
  t.colorSpace = THREE.SRGBColorSpace;
  t.anisotropy = 8;
  return t;
}

// キーボードの浅い凹み
const well = new THREE.Mesh(
  new RoundedBoxGeometry(KB_U * PITCH + 0.5, 0.3, KB_ROWS * PITCH + 0.5, 2, 0.02),
  grooveMat
);
well.position.set(0, TOP - 0.1, kbZ0 + (KB_ROWS - 1) * PITCH / 2);
body.add(well);

const keyGroup = new THREE.Group();
keyGroup.position.set(0, TOP + CAP_H / 2 - 0.12, kbZ0);
body.add(keyGroup);

const capMat = new THREE.MeshStandardMaterial({ color: '#f6f5f1', roughness: 0.48 });
const modMat = new THREE.MeshStandardMaterial({ color: '#e2e0da', roughness: 0.5 });

const keys = [];          // { def, mesh, down }
const keyByCode = new Map();
const pickables = [];

ROWS.forEach((row, r) => {
  const totalU = row.reduce((s, k) => s + k[2], 0);
  let x = -totalU * PITCH / 2;
  row.forEach((k) => {
    const [code, label, wU, keyCode, ch, shiftCh] = k;
    const def = { code, label, keyCode, char: ch, shiftChar: shiftCh, key: KEY_NAMES[code] };
    const w = wU * PITCH - (PITCH - CAP);
    const mat = ch !== undefined && code !== 'Space' ? capMat : modMat;
    const mesh = new THREE.Mesh(new RoundedBoxGeometry(w, CAP_H, CAP, 2, 0.025), mat);
    mesh.position.set(x + (wU * PITCH) / 2, 0, r * PITCH);
    mesh.castShadow = true;
    mesh.receiveShadow = true;
    mesh.userData.keyIndex = keys.length;

    if (label) {
      const top = new THREE.Mesh(
        new THREE.PlaneGeometry(w * 0.9, CAP * 0.9),
        new THREE.MeshBasicMaterial({ map: keyLegend(def, w / CAP), transparent: true, depthWrite: false })
      );
      top.rotation.x = -Math.PI / 2;
      top.position.y = CAP_H / 2 + 0.005;
      mesh.add(top);
    }

    keyGroup.add(mesh);
    const entry = { def, mesh, down: false };
    keys.push(entry);
    keyByCode.set(code, entry);
    pickables.push(mesh);
    x += wU * PITCH;
  });
});

function setKeyVisual(entry, down) {
  entry.mesh.position.y = down ? -TRAVEL : 0;
}

// ---------------------------------------------------------------- SDL input bridge

const sticky = { shift: false, ctrl: false };

function fireKey(type, def) {
  const shift = sticky.shift;
  const ch = def.char !== undefined ? (shift ? def.shiftChar : def.char) : undefined;
  const key = def.key ?? ch;
  const ev = new KeyboardEvent(type, {
    key, code: def.code, bubbles: true, cancelable: true,
    shiftKey: shift, ctrlKey: sticky.ctrl, location: def.code.endsWith('Right') && def.key === 'Shift' ? 2 : 0,
  });
  const charCode = type === 'keypress' ? ch.charCodeAt(0) : 0;
  const keyCode = type === 'keypress' ? charCode : def.keyCode;
  Object.defineProperty(ev, 'keyCode', { get: () => keyCode });
  Object.defineProperty(ev, 'which', { get: () => keyCode });
  Object.defineProperty(ev, 'charCode', { get: () => charCode });
  window.dispatchEvent(ev);
}

function virtualDown(entry) {
  const { def } = entry;
  if (MODIFIERS.has(def.code)) {
    // Shift / Ctrl はトグル式（次のキーで自動解除）
    const which = def.key === 'Shift' ? 'shift' : 'ctrl';
    sticky[which] = !sticky[which];
    for (const e of keys) if (MODIFIERS.has(e.def.code) && (e.def.key === 'Shift') === (which === 'shift')) {
      e.down = sticky[which];
      setKeyVisual(e, e.down);
    }
    fireKey(sticky[which] ? 'keydown' : 'keyup', def);
    return;
  }
  entry.down = true;
  setKeyVisual(entry, true);
  fireKey('keydown', def);
  if (def.char !== undefined && !sticky.ctrl) fireKey('keypress', def);
}

function virtualUp(entry) {
  if (MODIFIERS.has(entry.def.code)) return;
  entry.down = false;
  setKeyVisual(entry, false);
  fireKey('keyup', entry.def);
  for (const which of ['shift', 'ctrl']) {
    if (!sticky[which]) continue;
    sticky[which] = false;
    const mod = keyByCode.get(which === 'shift' ? 'ShiftLeft' : 'ControlLeft');
    fireKey('keyup', mod.def);
    for (const e of keys) if (MODIFIERS.has(e.def.code) && (e.def.key === 'Shift') === (which === 'shift')) {
      e.down = false;
      setKeyVisual(e, false);
    }
  }
}

// 物理キーボードは SDL が直接受ける。ここでは 3D キーを連動させるだけ
addEventListener('keydown', (e) => {
  if (!e.isTrusted) return;
  const k = keyByCode.get(e.code === 'ControlRight' ? 'ControlLeft' : e.code);
  if (k) setKeyVisual(k, true);
});
addEventListener('keyup', (e) => {
  if (!e.isTrusted) return;
  const k = keyByCode.get(e.code === 'ControlRight' ? 'ControlLeft' : e.code);
  if (k && !k.down) setKeyVisual(k, false);
});

function fireMouse(type, uv, src) {
  const rect = sdlCanvas.getBoundingClientRect();
  const clientX = rect.left + uv.x * rect.width;
  const clientY = rect.top + (1 - uv.y) * rect.height;
  if (type === 'wheel') {
    sdlCanvas.dispatchEvent(new WheelEvent('wheel', {
      clientX, clientY, deltaX: src.deltaX, deltaY: src.deltaY, deltaMode: src.deltaMode,
      bubbles: true, cancelable: true,
    }));
    return;
  }
  sdlCanvas.dispatchEvent(new MouseEvent(type, {
    clientX, clientY, screenX: clientX, screenY: clientY,
    button: src.button ?? 0, buttons: src.buttons ?? 0,
    bubbles: true, cancelable: true, view: window,
  }));
}

// ---------------------------------------------------------------- pointer picking

const raycaster = new THREE.Raycaster();
const ndc = new THREE.Vector2();
let pressedKey = null;
let screenDrag = false;

function pick(e, targets) {
  ndc.set((e.clientX / innerWidth) * 2 - 1, -(e.clientY / innerHeight) * 2 + 1);
  raycaster.setFromCamera(ndc, camera);
  return raycaster.intersectObjects(targets, false)[0];
}

stage.addEventListener('pointerdown', (e) => {
  e.preventDefault();
  stage.setPointerCapture(e.pointerId);
  const hitScreen = pick(e, [screen]);
  if (hitScreen) {
    screenDrag = true;
    fireMouse('mousemove', hitScreen.uv, e);
    fireMouse('mousedown', hitScreen.uv, e);
    return;
  }
  const hitKey = pick(e, pickables);
  if (hitKey) {
    pressedKey = keys[hitKey.object.userData.keyIndex];
    virtualDown(pressedKey);
  }
});

stage.addEventListener('pointermove', (e) => {
  const hitScreen = pick(e, [screen]);
  if (hitScreen) fireMouse('mousemove', hitScreen.uv, e);
  stage.style.cursor = hitScreen ? 'none' : pick(e, pickables) ? 'pointer' : 'default';
});

function endPointer(e) {
  if (screenDrag) {
    screenDrag = false;
    const hit = pick(e, [screen]);
    const uv = hit ? hit.uv : new THREE.Vector2(0, 0);
    fireMouse('mouseup', uv, e);
  }
  if (pressedKey) {
    virtualUp(pressedKey);
    pressedKey = null;
  }
}
stage.addEventListener('pointerup', endPointer);
stage.addEventListener('pointercancel', endPointer);

stage.addEventListener('wheel', (e) => {
  e.preventDefault();
  const hit = pick(e, [screen]);
  if (hit) fireMouse('wheel', hit.uv, e);
}, { passive: false });

// ---------------------------------------------------------------- layout / loop

const VIEW_DIR = new THREE.Vector3(0, 1.25, 0.78).normalize();
const FIT_MARGIN = 0.9; // NDC 上で筐体が占める最大範囲

scene.updateMatrixWorld();
const bounds = new THREE.Box3().setFromObject(body);
const corners = [];
for (const x of [bounds.min.x, bounds.max.x])
  for (const y of [bounds.min.y, bounds.max.y])
    for (const z of [bounds.min.z, bounds.max.z]) corners.push(new THREE.Vector3(x, y, z));

function placeCamera(focus, dist) {
  camera.position.copy(focus).addScaledVector(VIEW_DIR, dist);
  camera.lookAt(focus);
  camera.updateMatrixWorld();
}

// 筐体の 8 頂点を投影し、画面中央に収まるまで注視点と距離を詰める
function fitCamera() {
  const focus = bounds.getCenter(new THREE.Vector3());
  let dist = 60;
  const p = new THREE.Vector3();
  const right = new THREE.Vector3(), up = new THREE.Vector3();
  const tanV = Math.tan(THREE.MathUtils.degToRad(camera.fov) / 2);
  for (let i = 0; i < 24; i++) {
    placeCamera(focus, dist);
    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const c of corners) {
      p.copy(c).project(camera);
      minX = Math.min(minX, p.x); maxX = Math.max(maxX, p.x);
      minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y);
    }
    const cx = (minX + maxX) / 2, cy = (minY + maxY) / 2;
    const ext = Math.max((maxX - minX) / 2, (maxY - minY) / 2) / FIT_MARGIN;
    right.setFromMatrixColumn(camera.matrixWorld, 0);
    up.setFromMatrixColumn(camera.matrixWorld, 1);
    const halfH = dist * tanV;
    focus.addScaledVector(right, cx * halfH * camera.aspect).addScaledVector(up, cy * halfH);
    dist *= ext;
  }
  placeCamera(focus, dist);
}

function resize() {
  const w = innerWidth, h = innerHeight;
  renderer.setSize(w, h);
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  fitCamera();
}
addEventListener('resize', resize);
resize();

const clock = new THREE.Clock();
renderer.setAnimationLoop(() => {
  const t = clock.getElapsedTime();
  // SDL がキャンバスを作り直す／サイズを変えたらテクスチャを確保し直す
  if (sdlCanvas.width !== texSize[0] || sdlCanvas.height !== texSize[1]) {
    texSize[0] = sdlCanvas.width;
    texSize[1] = sdlCanvas.height;
    screenTex.dispose();
  }
  screenTex.needsUpdate = true;
  // 待機中の LED はゆっくり明滅
  const pulse = 0.75 + 0.25 * Math.sin(t * 2.0);
  led.material.color.set(AMBER).multiplyScalar(pulse);
  ledGlow.intensity = 0.6 * pulse;
  renderer.render(scene, camera);
});

// デバッグ用（DevTools から参照）
window.harbour = { scene, camera, screen, keys };
