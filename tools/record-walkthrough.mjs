#!/usr/bin/env node
// Optional media tooling: macOS say, Chrome, FFmpeg and FFprobe. No network.
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { mkdir, writeFile, stat } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { browser, delay } from "./browser.mjs";
import { chapters } from "./walkthrough-script.mjs";
const run = promisify(execFile), workspace = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const scratch = join(workspace, "scratch/walkthrough"), assets = join(workspace, "docs/assets"), frames = join(scratch, "frames");
await Promise.all([mkdir(frames, { recursive: true }), mkdir(assets, { recursive: true })]);
const ffmpeg = process.env.FFMPEG || "ffmpeg", ffprobe = process.env.FFPROBE || "ffprobe";
const voice = process.env.SCHEMATIC_VOICE || "Samantha (English (US))";
for (let i = 0; i < chapters.length; i++) {
  const chapter = chapters[i]; chapter.audio = join(scratch, `${String(i).padStart(2, "0")}.aiff`);
  await run("/usr/bin/say", ["-v", voice, "-r", "166", "-o", chapter.audio, chapter.narration], { timeout: 60000 });
  const { stdout } = await run(ffprobe, ["-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", chapter.audio]);
  chapter.planned = Math.max(chapter.min, Number(stdout.trim()) + .8);
}
console.log(`Narration prepared: ${chapters.length} chapters, about ${Math.round(chapters.reduce((t, c) => t + c.planned, 0))} seconds.`);
const b = await browser({ width: 1440, height: 1008 });
const images = [], writes = [], acknowledgements = [];
let draining, recording = false, started = 0, sequence = 0;
const evaluate = (code) => b.evaluate(code);
async function caption(chapter, index) {
  await evaluate(`(()=>{const n=document.getElementById('film-notes');n.querySelector('[data-film-count]').textContent=${JSON.stringify(`${String(index + 1).padStart(2, "0")} / ${chapters.length}`)};n.querySelector('strong').textContent=${JSON.stringify(chapter.title)};n.querySelector('p').textContent=${JSON.stringify(chapter.subtitle)};n.hidePopover();n.showPopover();document.getElementById('film-cursor').hidePopover();document.getElementById('film-cursor').showPopover();})()`);
}
async function overlays() { await evaluate("for(const id of ['film-notes','film-cursor']){const el=document.getElementById(id);el.hidePopover();el.showPopover()}"); }
async function point(selector, { click = false, hover = 250 } = {}) {
  const offscreen = await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)return false;const r=e.getBoundingClientRect();return !!e.closest('#viewport')&&(r.top<85||r.bottom>875);})()`);
  if (offscreen) { await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)}),v=document.querySelector('#viewport');Schematic.scrollTo(v.scrollTop+e.getBoundingClientRect().top-v.getBoundingClientRect().top-Math.max(32,(v.clientHeight-e.getBoundingClientRect().height)*.4));})()`); await delay(950); }
  const rect = await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw Error('Missing video target: '+${JSON.stringify(selector)});const r=e.getBoundingClientRect();return{x:r.left+r.width/2,y:r.top+r.height/2};})()`);
  await evaluate(`document.querySelector('#film-cursor').style.transform='translate(${rect.x}px,${rect.y}px)'`);
  await delay(230); await b.send("Input.dispatchMouseEvent", { type: "mouseMoved", ...rect });
  if (click) {
    await evaluate("document.getElementById('film-cursor').classList.add('is-press')");
    await b.send("Input.dispatchMouseEvent", { type: "mousePressed", button: "left", clickCount: 1, ...rect });
    await delay(80); await b.send("Input.dispatchMouseEvent", { type: "mouseReleased", button: "left", clickCount: 1, ...rect });
    await evaluate("document.getElementById('film-cursor').classList.remove('is-press')"); await overlays();
  }
  await delay(hover);
}
async function navigate(id) { await evaluate(`Schematic.navigate(${JSON.stringify(id)})`); await delay(650); }
async function scroll(selector, offset = 20) {
  await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});if(!e)throw Error('Missing scroll target');const v=document.querySelector('#viewport');Schematic.scrollTo(v.scrollTop+e.getBoundingClientRect().top-v.getBoundingClientRect().top-${offset});})()`); await delay(1000);
}
async function fill(selector, value) {
  await point(selector, { click: true, hover: 50 });
  await evaluate(`(()=>{const e=document.querySelector(${JSON.stringify(selector)});e.value='';e.dispatchEvent(new Event('input',{bubbles:true}));})()`);
  for (const char of value) { await b.send("Input.insertText", { text: char }); await delay(55); }
  await delay(300);
}
async function key(name, code, virtual) { await b.send("Input.dispatchKeyEvent", { type: "keyDown", key: name, code, windowsVirtualKeyCode: virtual }); await b.send("Input.dispatchKeyEvent", { type: "keyUp", key: name, code, windowsVirtualKeyCode: virtual }); await delay(250); }
const actions = {
  async welcome() { await delay(1600); await point('.ov-cta a', { hover: 700 }); await point('#tune-btn', { hover: 500 }); await b.screenshot(join(assets, "walkthrough-poster.png")); },
  async guide() {
    await evaluate("Schematic.onboarding.open()"); await overlays(); await delay(1800);
    await point('[data-guide-next]', { click: true, hover: 1700 });
    const color = await evaluate("!!document.querySelector('#schematic-guide [data-value=mono]')");
    if (color) { await point('#schematic-guide [data-value=mono]', { click: true, hover: 700 }); await point('#schematic-guide [data-value=color]', { click: true, hover: 500 }); }
    await point('[data-guide-next]', { click: true, hover: 1100 });
    if (await evaluate("!!document.querySelector('[data-guide-motion]')")) await point('[data-guide-motion]', { click: true, hover: 1000 });
    await point('[data-guide-next]', { click: true, hover: 1500 }); await evaluate("Schematic.onboarding.close()"); await overlays();
  },
  async foundations() { await navigate('foundations'); await point('[data-token=accent]', { hover: 500 }); await scroll('#foundations-type'); await delay(1400); await scroll('#foundations-space'); await delay(1200); await point('#tune-btn', { click: true }); await point('#palette-seg [data-value=mono]', { click: true, hover: 1200 }); await point('#palette-seg [data-value=color]', { click: true, hover: 700 }); await point('#tune-btn', { click: true }); },
  async motion() { await navigate('motion'); await scroll('#motion-curves'); await point('#motion-play', { click: true, hover: 1500 }); await evaluate("(()=>{const e=document.querySelector('#motion-curve');e.value='springSnappy';e.dispatchEvent(new Event('change',{bubbles:true}))})()"); await delay(1300); await scroll('#motion-springs'); await evaluate("(()=>{const e=document.querySelector('#motion-damping');e.value='14';e.dispatchEvent(new Event('input',{bubbles:true}))})()"); await delay(1700); },
  async icons() { await navigate('icons'); await scroll('#ic-gallery'); await point('#icons-wave', { click: true, hover: 1700 }); await fill('#icons-q','bell'); await point('[data-name=bell]', { hover: 1300 }); await fill('#icons-q',''); await delay(700); },
  async components() { await navigate('components'); await scroll('#cp-states'); await point('[data-async=save]', { click: true, hover: 1500 }); await point('[data-async=retry]', { click: true, hover: 1300 }); await point('[data-async=retry]', { click: true, hover: 1300 }); await point('[data-favourite]', { click: true, hover: 700 }); await point('[data-hold]', { click: true, hover: 700 }); await point('[data-hold]', { hover: 100 }); const r=await evaluate("(()=>{const r=document.querySelector('[data-hold]').getBoundingClientRect();return{x:r.left+r.width/2,y:r.top+r.height/2}})()"); await b.send('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...r});await delay(1650);await b.send('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...r});await delay(500); },
  async forms() { await navigate('forms'); await scroll('#forms-email-demo'); await fill('#forms-email','alex@'); await delay(600); await fill('#forms-email','alex@example.com'); await delay(700); await scroll('#forms-security'); await fill('#forms-password','Schematic-2026!'); await delay(800); await scroll('#forms-otp-demo'); await evaluate("(()=>{const el=document.querySelector('#forms-otp-slots input'),d=new DataTransfer();d.setData('text','123456');el.dispatchEvent(new ClipboardEvent('paste',{clipboardData:d,bubbles:true,cancelable:true}));document.querySelector('#forms-otp-demo').requestSubmit()})()"); await delay(1300); },
  async charts() { await navigate('data'); await scroll('[data-main-chart]'); await delay(1500); await scroll('#dt-overview'); await point('[data-control=period] [data-value="7"]',{click:true,hover:300}); await scroll('[data-main-chart]'); await delay(1400); await point('[data-chart-hit]',{hover:1000}); await evaluate("document.querySelector('[data-control=period] [data-value=\"30\"]').click()"); await delay(1600); },
  async inspect() { await scroll('#dt-comparisons'); await point('[data-bar-group="2"]',{hover:800}); await point('[data-rank-inspect="0"]',{hover:800}); await point('[data-control=bars] [data-value=stacked]',{click:true,hover:1100}); await scroll('#dt-composition'); await point('[data-device="1"]',{hover:1000}); await point('[data-goal="0"]',{click:true,hover:1100}); },
  async table() { await scroll('#dt-records'); await fill('.pg-data [data-search]','Orbit Supply'); await point('[data-account="SC-1026"] td.dt-money',{click:true,hover:1100}); await point('[data-clear-selection]',{click:true,hover:500}); await fill('.pg-data [data-search]',''); await point('[data-table-sort=name]',{click:true,hover:700}); },
  async overlays() { await navigate('overlays'); await scroll('[data-ov-open=project]'); await point('[data-ov-open=project]',{click:true,hover:1500}); await key('Escape','Escape',27); await delay(500); await scroll('#ov-surfaces'); await point('[data-ov-open=drawer]',{click:true,hover:1400}); await key('Escape','Escape',27); await point('[data-ov-open=sheet]',{click:true,hover:1400}); await key('Escape','Escape',27); },
  async themes() { await navigate('overview'); await point('#theme-btn',{click:true,hover:1100}); await point('#tune-btn',{click:true}); await point('#palette-seg [data-value=mono]',{click:true,hover:900}); await point('#tune-btn',{click:true}); await point('#theme-btn',{click:true,hover:1100}); await point('[data-open-cmdk]',{click:true,hover:700}); await fill('.cmdk-input','icons'); await delay(500); await key('Enter','Enter',13); await delay(800); },
  async finish() { await evaluate("Schematic.setPalette('color')"); await navigate('overview'); await evaluate("document.querySelector('#film-cursor').style.opacity='0'"); await delay(1200); },
};
try {
  await b.open(pathToFileURL(join(workspace, "dist/schematic.html")).href);
  await evaluate(`(()=>{localStorage.clear();Schematic.setMotion('full');Schematic.setPalette('color');document.querySelector('.app').style.setProperty('height','900px','important');const s=document.createElement('style');s.textContent=\`
  #film-notes{position:fixed;inset:auto 0 0;margin:0;width:100%;height:108px;border:0;border-top:1px solid #393a34;padding:19px 44px;background:#171914;color:#fafaf6;display:flex;gap:28px;align-items:center;max-width:none;pointer-events:none;font:14px -apple-system,BlinkMacSystemFont,sans-serif;box-sizing:border-box}
  #film-notes [data-film-count]{color:#a9b4a4;font:13px ui-monospace,monospace;border-right:1px solid #43473e;padding-right:28px}
  #film-notes strong{display:block;font-size:23px;font-weight:600;letter-spacing:-.3px;line-height:1.4}#film-notes p{margin:3px 0 0;color:#bac0b5;font-size:14px}
  #film-notes .film-brand{margin-left:auto;font:12px ui-monospace,monospace;color:#b8c4b0;letter-spacing:2px}
  #film-cursor{position:fixed;inset:0 auto auto 0;margin:0;border:0;background:none;padding:0;width:26px;height:32px;overflow:visible;pointer-events:none;transform:translate(1200px,600px);transition:transform 220ms cubic-bezier(.22,1,.36,1);max-width:none}
  #film-cursor svg{width:25px;height:31px;filter:drop-shadow(0 2px 2px #0004);transition:scale 120ms ease-out;transform-origin:top left}#film-cursor.is-press svg{scale:.85}
  \`;document.head.append(s);const n=document.createElement('aside');n.id='film-notes';n.popover='manual';n.setAttribute('aria-hidden','true');n.innerHTML='<span data-film-count>01 / 13</span><div><strong></strong><p></p></div><span class="film-brand">SCHEMATIC</span>';document.body.append(n);n.showPopover();const c=document.createElement('div');c.id='film-cursor';c.popover='manual';c.setAttribute('aria-hidden','true');c.innerHTML='<svg viewBox="0 0 25 31"><path d="M3 2v23l6-6 5 10 4-2-5-10h9Z" fill="white" stroke="#20231e" stroke-width="1.5" stroke-linejoin="round"/></svg>';document.body.append(c);c.showPopover()})()`);
  if (!await evaluate("!!Schematic.onboarding?.open")) throw new Error("Build the onboarding before recording the walkthrough");
  await caption(chapters[0], 0); await delay(1000);
  b.events.splice(0); recording = true;
  const drain = () => {
    for (const event of b.events.splice(0)) if (event.method === "Page.screencastFrame") {
      const p=event.params; acknowledgements.push(b.send("Page.screencastFrameAck",{sessionId:p.sessionId}));
      if (!recording) continue;
      const file=join(frames,`${String(sequence++).padStart(6,"0")}.jpg`); images.push({file,t:p.metadata.timestamp}); writes.push(writeFile(file,Buffer.from(p.data,"base64")));
    }
  };
  draining=setInterval(drain,12); started=performance.now();
  await b.send("Page.startScreencast",{format:"jpeg",quality:92,maxWidth:1440,maxHeight:1008,everyNthFrame:2});
  for(let i=0;i<chapters.length;i++){
    const c=chapters[i];c.start=(performance.now()-started)/1000;await caption(c,i);console.log(`Recording ${i+1}/${chapters.length}: ${c.title}`);
    const tick=performance.now();await actions[c.id]();await delay(Math.max(0,c.planned*1000-(performance.now()-tick)));c.duration=(performance.now()-started)/1000-c.start;
  }
  recording=false;await b.send("Page.stopScreencast");drain();clearInterval(draining);await Promise.all([...writes,...acknowledgements]);
  if(b.failures.length)throw Error(`Recording browser errors: ${b.failures.join('; ')}`);
} finally { clearInterval(draining);await b.close(); }
const total=chapters.reduce((sum,c)=>sum+c.duration,0);
let concat="ffconcat version 1.0\n";
images.forEach((f,i)=>{const duration=i<images.length-1?Math.max(.001,images[i+1].t-f.t):.1;concat+=`file '${f.file.replaceAll("'","'\\''")}'\nduration ${duration.toFixed(6)}\n`;});concat+=`file '${images.at(-1).file}'\n`;
const concatFile=join(scratch,'frames.ffconcat');await writeFile(concatFile,concat);
const audioArgs=chapters.flatMap(c=>['-i',c.audio]);const pads=chapters.map((c,i)=>`[${i}:a]apad,atrim=duration=${c.duration.toFixed(6)},asetpts=PTS-STARTPTS,afade=t=in:d=0.035,afade=t=out:st=${Math.max(0,c.duration-.15).toFixed(3)}:d=0.15[a${i}]`).join(';');
const audio=join(scratch,'narration.wav');await run(ffmpeg,['-hide_banner','-loglevel','error','-y',...audioArgs,'-filter_complex',pads+';'+chapters.map((_,i)=>`[a${i}]`).join('')+`concat=n=${chapters.length}:v=0:a=1[out]`,'-map','[out]','-c:a','pcm_s16le',audio],{timeout:60000});
const movie=join(assets,'schematic-walkthrough.mp4');console.log(`Encoding ${images.length} real browser frames, ${total.toFixed(1)} seconds.`);
await run(ffmpeg,['-hide_banner','-loglevel','error','-y','-f','concat','-safe','0','-i',concatFile,'-i',audio,'-vf','tpad=stop_mode=clone:stop_duration=2,fps=30,format=yuv420p','-c:v','libx264','-preset','medium','-crf','20','-c:a','aac','-ar','48000','-b:a','192k','-af','loudnorm=I=-16:TP=-1.5:LRA=11','-movflags','+faststart','-t',total.toFixed(3),movie],{timeout:300000,maxBuffer:8*1024*1024});
const stamp=(n)=>{const h=Math.floor(n/3600),m=Math.floor(n%3600/60),s=(n%60).toFixed(3).padStart(6,'0');return `${String(h).padStart(2,'0')}:${String(m).padStart(2,'0')}:${s}`;};
let vtt='WEBVTT\n\n';let cue=0;
for(const c of chapters){const sentences=c.narration.match(/[^.!?]+[.!?]+/g)||[c.narration];const words=sentences.map(s=>s.trim().split(/\s+/).length),sum=words.reduce((a,b)=>a+b,0);let t=c.start;for(let i=0;i<sentences.length;i++){const d=(c.duration-.5)*words[i]/sum;vtt+=`${++cue}\n${stamp(t)} --> ${stamp(t+d)}\n${sentences[i].trim()}\n\n`;t+=d;}}
await writeFile(join(assets,'walkthrough.vtt'),vtt.trimEnd()+'\n');
await writeFile(join(assets,'walkthrough-chapters.json'),JSON.stringify(chapters.map(({id,title,subtitle,start,duration})=>({id,title,subtitle,start:+start.toFixed(3),duration:+duration.toFixed(3)})),null,2)+'\n');
const transcript='# Walkthrough transcript\n\nThe video records the actual offline interface. Narration uses the local macOS\nSamantha voice; no external footage or user information is included.\n\n'+chapters.map(c=>`## ${Math.floor(c.start/60)}:${String(Math.floor(c.start%60)).padStart(2,'0')} — ${c.title}\n\n${c.narration}\n`).join('\n');await writeFile(join(workspace,'docs/WALKTHROUGH.md'),transcript);
await writeFile(join(scratch,'metadata.json'),JSON.stringify({seconds:total,frames:images.length,bytes:(await stat(movie)).size,chapters:chapters.map(({id,start,duration})=>({id,start,duration}))},null,2));
console.log(`Saved ${movie} (${((await stat(movie)).size/1024/1024).toFixed(1)} MiB), captions, chapters and transcript.`);
