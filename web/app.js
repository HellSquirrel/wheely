// Wheely remote over Web Bluetooth. Speaks the same Nordic UART Service and the same
// Bluefruit Connect control-pad packets as the phone app, so the firmware is unchanged.

const NUS = '6e400001-b5a3-f393-e0a9-e50e24dcca9e';
const NUS_RX = '6e400002-b5a3-f393-e0a9-e50e24dcca9e'; // we write here
const NUS_TX = '6e400003-b5a3-f393-e0a9-e50e24dcca9e'; // wheely notifies here

const $ = (id) => document.getElementById(id);
const ui = {
  connect: $('connect'), dot: $('dot'), state: $('state'), status: $('status'),
  distance: $('distance'), blocked: $('blocked'), log: $('log'), install: $('install'),
  boot: $('boot'), stop: $('stop'), unsupported: $('unsupported'),
  dirs: [...document.querySelectorAll('.pad .dir')],
  speeds: [...document.querySelectorAll('.speeds button')],
};

let device = null, rx = null, tx = null;
let queue = Promise.resolve();   // GATT allows one operation at a time
let held = null;                 // button code currently held ('5'..'8')
let lastSpeed = '2';

function log(msg) {
  const t = new Date().toLocaleTimeString([], { hour12: false });
  ui.log.textContent = (`${t} ${msg}\n` + ui.log.textContent).slice(0, 4000);
  console.log('[wheely]', msg);
}

function setState(kind, text) {
  ui.dot.className = 'dot ' + (kind === 'on' ? 'on' : kind === 'busy' ? 'busy' : '');
  ui.state.textContent = text;
  const connected = kind === 'on';
  ui.connect.textContent = connected ? 'Disconnect' : (device ? 'Reconnect' : 'Connect');
  for (const b of [...ui.dirs, ...ui.speeds, ui.stop, ui.boot]) b.disabled = !connected;
}

// ---------------------------------------------------------------- protocol

const enc = new TextEncoder();

/** Bluefruit control-pad packet: '!' 'B' button state checksum(~sum). */
function padPacket(button, pressed) {
  const bytes = enc.encode(`!B${button}${pressed ? '1' : '0'}`);
  const sum = bytes.reduce((a, b) => (a + b) & 0xff, 0);
  return Uint8Array.of(...bytes, (~sum) & 0xff);
}

function send(bytes) {
  if (!rx) return Promise.resolve();
  queue = queue
    .then(() => rx.writeValueWithoutResponse(bytes))
    .catch((e) => log(`write failed: ${e.message}`));
  return queue;
}

function press(button) {
  if (held === button) return;
  if (held) release(held);
  held = button;
  ui.dirs.find((b) => b.dataset.btn === button)?.classList.add('pressed');
  send(padPacket(button, true));
}

function release(button) {
  if (held !== button) return;
  held = null;
  ui.dirs.find((b) => b.dataset.btn === button)?.classList.remove('pressed');
  send(padPacket(button, false));
}

function stopAll() {
  if (held) release(held);
  send(enc.encode('s'));
}

function setSpeed(n) {
  lastSpeed = n;
  ui.speeds.forEach((b) => b.classList.toggle('active', b.dataset.speed === n));
  send(padPacket(n, true));
  send(padPacket(n, false));
}

// ---------------------------------------------------------------- telemetry

const dec = new TextDecoder();
function onNotify(ev) {
  const line = dec.decode(ev.target.value).trim(); // e.g. "Forward 523mm !"
  if (!line) return;
  const blocked = line.endsWith('!');
  const m = line.match(/(\d+)mm/);
  ui.distance.textContent = m ? `${(m[1] / 10).toFixed(0)} cm` : line.replace(/^\S+\s*/, '') || '—';
  ui.blocked.hidden = !blocked;
  ui.dirs.find((b) => b.dataset.btn === '5')?.classList.toggle('blocked', blocked);
  ui.status.textContent = line;
}

// ---------------------------------------------------------------- connection

async function connect() {
  try {
    if (!device) {
      setState('busy', 'choosing…');
      device = await navigator.bluetooth.requestDevice({
        filters: [{ namePrefix: 'wheely' }],
        optionalServices: [NUS],
      });
      device.addEventListener('gattserverdisconnected', onDisconnected);
    }
    setState('busy', 'connecting…');
    const server = await device.gatt.connect();
    const service = await server.getPrimaryService(NUS);
    rx = await service.getCharacteristic(NUS_RX);
    tx = await service.getCharacteristic(NUS_TX);
    await tx.startNotifications();
    tx.addEventListener('characteristicvaluechanged', onNotify);
    setState('on', device.name || 'connected');
    ui.status.textContent = 'Connected. Hold an arrow to drive.';
    log(`connected to ${device.name}`);
    setSpeed(lastSpeed);
  } catch (e) {
    log(`connect failed: ${e.message}`);
    setState('off', 'not connected');
    ui.status.textContent = e.name === 'NotFoundError' ? 'No device chosen.' : e.message;
    if (e.name === 'NotFoundError') device = null;
  }
}

function onDisconnected() {
  rx = tx = null;
  held = null;
  ui.dirs.forEach((b) => b.classList.remove('pressed'));
  setState('off', 'disconnected');
  ui.status.textContent = 'Connection lost. Wheely stops on its own.';
  log('disconnected');
}

function disconnect() {
  stopAll();
  queue.then(() => device?.gatt.connected && device.gatt.disconnect());
}

// ---------------------------------------------------------------- UI wiring

ui.connect.addEventListener('click', () => (device?.gatt.connected ? disconnect() : connect()));
ui.stop.addEventListener('click', stopAll);
ui.boot.addEventListener('click', () => { log('sent boot'); send(enc.encode('boot\n')); });
ui.speeds.forEach((b) => b.addEventListener('click', () => setSpeed(b.dataset.speed)));

for (const b of ui.dirs) {
  const code = b.dataset.btn;
  b.addEventListener('pointerdown', (ev) => { ev.preventDefault(); b.setPointerCapture(ev.pointerId); press(code); });
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
    b.addEventListener(type, () => release(code));
  }
  b.addEventListener('contextmenu', (ev) => ev.preventDefault());
}

const KEYS = {
  ArrowUp: '5', KeyW: '5', ArrowDown: '6', KeyS: '6', ArrowLeft: '7', KeyA: '7', ArrowRight: '8', KeyD: '8',
};
document.addEventListener('keydown', (ev) => {
  if (ev.repeat || ev.target.tagName === 'INPUT') return;
  if (KEYS[ev.code]) { ev.preventDefault(); press(KEYS[ev.code]); }
  else if (ev.code === 'Space') { ev.preventDefault(); stopAll(); }
  else if (/^Digit[1-4]$/.test(ev.code)) setSpeed(ev.code.slice(-1));
});
document.addEventListener('keyup', (ev) => { if (KEYS[ev.code]) release(KEYS[ev.code]); });

// If the page is hidden mid-press (call comes in, app switch), let go.
document.addEventListener('visibilitychange', () => { if (document.hidden) stopAll(); });
window.addEventListener('blur', () => { if (held) stopAll(); });

// ---------------------------------------------------------------- PWA bits

if (!('bluetooth' in navigator)) {
  ui.unsupported.hidden = false;
  ui.unsupported.textContent = 'This browser has no Web Bluetooth. Use Chrome or Edge on Android, Windows, macOS or ChromeOS. On iPhone use the "Bluefy" browser.';
  ui.connect.disabled = true;
} else if (!window.isSecureContext) {
  ui.unsupported.hidden = false;
  ui.unsupported.textContent = 'Web Bluetooth needs HTTPS (or localhost).';
  ui.connect.disabled = true;
}
setState('off', 'not connected');

let installPrompt = null;
window.addEventListener('beforeinstallprompt', (ev) => {
  ev.preventDefault();
  installPrompt = ev;
  ui.install.hidden = false;
});
ui.install.addEventListener('click', async () => {
  if (!installPrompt) return;
  installPrompt.prompt();
  await installPrompt.userChoice;
  installPrompt = null;
  ui.install.hidden = true;
});

if ('serviceWorker' in navigator) {
  navigator.serviceWorker.register('sw.js').catch((e) => log(`sw: ${e.message}`));
}
