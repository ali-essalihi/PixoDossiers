/* Pixo et les Dossiers - Frontend application (Tauri v2 + offline dev fallback) */
(() => {
  'use strict';

  const $ = s => document.querySelector(s);
  const esc = s => String(s == null ? '' : s).replace(/[&<>"]/g, c => ({
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;'
  }[c]));

  /* ---------- Emojis -> Twemoji SVG (<img class="emoji">) ---------- */
  const TWEMOJI = 'assets/emoji/';
  const EMOJI_RE = /(?:[\uD83C-\uD83E][\uDC00-\uDFFF]|[\u231A\u231B\u2328\u23CF\u23E9-\u23F3\u23F8-\u23FA\u25B6\u25C0\u2600-\u2604\u260E\u2611\u2614\u2615\u2618\u261D\u2620\u2622\u2623\u2626\u262A\u262E\u262F\u2638-\u263A\u2648-\u2653\u265F\u2660\u2663\u2665\u2666\u2668\u267B\u267E\u267F\u2692-\u2697\u2699\u269B\u269C\u26A0\u26A1\u26AA\u26AB\u26B0\u26B1\u26BD\u26BE\u26C4\u26C5\u26CE\u26D4\u26EA\u26F2-\u26F5\u26FA\u26FD\u2702\u2705\u2708-\u270D\u270F\u2712\u2714\u2716\u271D\u2721\u2728\u2733\u2734\u2744\u2747\u274C\u274E\u2753-\u2755\u2757\u2763\u2764\u2795-\u2797\u27A1\u27B0\u27BF\u2B05-\u2B07\u2B1B\u2B1C\u2B50\u2B55])(?:\uD83C[\uDFFB-\uDFFF])?\uFE0F?(?:\u200D(?:[\uD83C-\uD83E][\uDC00-\uDFFF]|[\u2640\u2642\u2695\u2696\u2708\u2764])\uFE0F?)*/g;

  function getEmojiImg(str) {
    return String(str).split(/(<[^>]*>)/).map((part, i) => i % 2 ? part : part.replace(EMOJI_RE, m => {
      let cps = Array.from(m).map(c => c.codePointAt(0).toString(16));
      if (cps.indexOf('200d') < 0) cps = cps.filter(c => c !== 'fe0f');
      return `<img class="emoji" draggable="false" alt="${m}" src="${TWEMOJI}${cps.join('-')}.svg">`;
    })).join('');
  }

  /* ---------- Flatten worlds -> global levels list ---------- */
  const DATA = window.PIXO_DATA || { avatars: ['🦊', '🐼', '🐸', '🦁', '🐙', '🐯'], worlds: [] };
  const AV = DATA.avatars;
  const LV = [];
  const W = {};
  DATA.worlds.forEach((w, wi) => {
    W[LV.length] = { title: `Monde ${wi + 1} : ${w.title}`, icon: w.icon || '📁' };
    (w.levels || []).forEach(l => {
      LV.push(Object.assign({}, l, { globalNum: LV.length + 1 }));
    });
  });

  const app = $('#app');
  const liveEl = $('#live-status');
  const bannerEl = $('#env-banner');

  function announce(msg) {
    if (liveEl) liveEl.textContent = msg;
  }

  function iconForKind(kind, name) {
    const k = (kind || '').toLowerCase();
    const n = (name || '').toLowerCase();
    if (k === 'folder') return '📁';
    if (k === 'png' || k === 'jpg' || k === 'image' || n.endsWith('.png') || n.endsWith('.jpg')) return '🖼️';
    if (k === 'mp4' || k === 'video' || n.endsWith('.mp4')) return '🎬';
    if (k === 'mp3' || k === 'audio' || n.endsWith('.mp3')) return '🎵';
    if (k === 'pdf' || n.endsWith('.pdf')) return '📄';
    return '📝';
  }

  /* ---------- Tauri v2 Bridge + Offline Dev Fallback ---------- */
  const hasTauri = () =>
    typeof window.__TAURI__ !== 'undefined' &&
    window.__TAURI__.core &&
    typeof window.__TAURI__.core.invoke === 'function';

  const invoke = (cmd, args) => window.__TAURI__.core.invoke(cmd, args || {});

  const Bridge = {
    async loadState() {
      if (hasTauri()) {
        try { return await invoke('load_state'); } catch (e) {}
      }
      try { return JSON.parse(localStorage.getItem('pixo_dossiers')); } catch (e) { return null; }
    },
    async saveState(st) {
      try { localStorage.setItem('pixo_dossiers', JSON.stringify(st)); } catch (e) {}
      if (hasTauri()) {
        try { await invoke('save_state', { data: st }); } catch (e) {}
      }
    },
    async checkEnv() {
      if (hasTauri()) {
        try { return await invoke('check_environment'); } catch (e) {}
      }
      return { ok: true, missing_payload: [], banner: '' };
    },
    async startTask(levelId, memberName, groupName) {
      if (hasTauri()) {
        try {
          return await invoke('start_task', {
            levelId,
            level_id: levelId,
            memberName: memberName || '',
            member_name: memberName || '',
            groupName: groupName || '',
            group_name: groupName || ''
          });
        } catch (e) {
          return { ok: false, error: 'Ferme les fenêtres ouvertes puis réessaie' };
        }
      }
      return { ok: true };
    },
    async verifyTask(levelId, memberName, groupName) {
      if (hasTauri()) {
        try {
          return await invoke('verify_task', {
            levelId,
            level_id: levelId,
            memberName: memberName || '',
            member_name: memberName || '',
            groupName: groupName || '',
            group_name: groupName || ''
          });
        } catch (e) {
          return { checks: [], score: 0, stars: 0, error: 'Erreur de vérification.' };
        }
      }
      return {
        checks: [
          { id: 'mock1', ok: true, label: 'Aperçu hors-ligne : vérification simulée réussie', hint: '', items: [] }
        ],
        score: 100,
        stars: 3
      };
    },
    async openFolder(key) {
      if (hasTauri()) {
        try { return await invoke('open_folder', { key }); } catch (e) {}
      }
      return { ok: true };
    }
  };

  /* ---------- Application State ---------- */
  let S = { v: 1, groups: {}, cur: null, wipe_confirmed: false };

  function save() {
    Bridge.saveState(S);
  }

  const G = () => (S.cur && S.groups[S.cur]) ? S.groups[S.cur] : null;
  const star = (g, li, m) => (g.prog && g.prog[li] && g.prog[li][m.n]) || 0;
  const nextMember = (g, li) => g.members.find(m => !(g.prog && g.prog[li] && g.prog[li][m.n] != null));
  const done = (g, li) => !nextMember(g, li);
  const unlocked = (g, li) => li === 0 || done(g, li - 1) || (g.start != null && li <= g.start);
  const mStars = (g, m) => LV.reduce((t, _, i) => t + star(g, i, m), 0);
  const gStars = g => g.members.reduce((t, m) => t + mStars(g, m), 0);
  const lvGroupStars = (g, li) => g.members.reduce((t, m) => t + star(g, li, m), 0);

  function applyStartLevel(g, startIdx) {
    g.start = startIdx;
    g.prog = g.prog || {};
    for (let i = 0; i < startIdx; i++) {
      g.prog[i] = g.prog[i] || {};
      g.members.forEach(m => {
        g.prog[i][m.n] = Math.max(g.prog[i][m.n] || 0, 3);
      });
    }
  }

  /* ---------- View Helper ---------- */
  const view = html => {
    window.onkeydown = window.onkeyup = null;
    app.innerHTML = getEmojiImg(html);
  };

  /* ---------- Home & Group Setup ---------- */
  function home() {
    const names = Object.keys(S.groups);
    view(`<div class="card">
      <h1>Bienvenue ! 👋</h1>
      <p>Aide Pixo à ranger les dossiers et les fichiers de l'ordinateur, en équipe !</p>
      <button type="button" class="btn" id="nw">➕ Créer un groupe</button>
    </div>` +
    (names.length ? `<div class="card">
      <h2>Reprendre un groupe</h2>
      <div>${names.map(n => `<button type="button" class="btn s" data-g="${esc(n)}">${esc(n)}</button>`).join('')}</div>
    </div>` : ''));
    $('#nw').onclick = () => setup(false);
    app.querySelectorAll('[data-g]').forEach(b => {
      b.onclick = () => {
        S.cur = b.dataset.g;
        save();
        map();
      };
    });
  }

  function setup(isFirst) {
    const usedAvatars = [];
    view(`<div class="card">
      <h1>${isFirst ? 'Bienvenue dans Pixo et les Dossiers ! 👋' : 'Nouveau groupe 👥'}</h1>
      <p>Crée un groupe de 2 à 6 élèves pour commencer.</p>
      <label>Nom du groupe
        <input id="gn" maxlength="20" placeholder="Ex : Les Explorateurs">
      </label>
      <h2>Les membres (2 à 6)</h2>
      <div id="ms"></div>
      <button type="button" class="btn ghost s" id="add">➕ Ajouter un membre</button>
      <h2>Niveau de départ</h2>
      <label>Le groupe commence au niveau :
        <select id="sl">${LV.map((l, i) => `<option value="${i}">Niveau ${l.globalNum} : ${esc(l.title)}${i === 0 ? ' (depuis le début)' : ''}</option>`).join('')}</select>
      </label>
      <small>Les niveaux avant celui-ci sont débloqués et comptés comme réussis.</small>
      <p class="hint" id="msg" role="alert"></p>
      <div>
        <button type="button" class="btn" id="go">🚀 Commencer</button>
        ${!isFirst && Object.keys(S.groups).length ? '<button type="button" class="btn ghost" id="cancel">⬅ Annuler</button>' : ''}
      </div>
    </div>`);

    const ms = $('#ms');
    const pickNextFreeAvatar = currentAv => {
      const taken = new Set(usedAvatars);
      const startIdx = currentAv ? (AV.indexOf(currentAv) + 1) % AV.length : 0;
      for (let k = 0; k < AV.length; k++) {
        const candidate = AV[(startIdx + k) % AV.length];
        if (!taken.has(candidate)) return candidate;
      }
      return AV[0];
    };

    const addMemberRow = () => {
      const i = ms.children.length;
      if (i >= 6) return;
      const av = pickNextFreeAvatar(null);
      usedAvatars.push(av);
      const row = document.createElement('div');
      row.className = 'member-row';
      row.dataset.idx = i;
      row.innerHTML = getEmojiImg(
        `<label><span class="av-disp">${av}</span> Prénom ${i + 1}<input maxlength="15" placeholder="Prénom ${i + 1}"></label>` +
        `<button type="button" class="btn ghost s reroll-btn" aria-label="Changer l'emoji du membre ${i + 1}" title="Changer l'emoji">🎲</button>`
      );
      const rerollBtn = row.querySelector('.reroll-btn');
      rerollBtn.onclick = () => {
        const idx = +row.dataset.idx;
        const nextAv = pickNextFreeAvatar(usedAvatars[idx]);
        usedAvatars[idx] = nextAv;
        row.querySelector('.av-disp').innerHTML = getEmojiImg(nextAv);
      };
      ms.appendChild(row);
      if (ms.children.length >= 6) $('#add').disabled = true;
    };

    addMemberRow();
    addMemberRow();
    $('#add').onclick = addMemberRow;
    if ($('#cancel')) $('#cancel').onclick = home;

    $('#go').onclick = () => {
      const gn = $('#gn').value.trim();
      const rows = [...ms.querySelectorAll('.member-row')];
      const members = [];
      rows.forEach((r, idx) => {
        const val = r.querySelector('input').value.trim();
        if (val) members.push({ n: val, av: usedAvatars[idx] || AV[idx % AV.length] });
      });

      if (!gn) return $('#msg').innerHTML = getEmojiImg('💡 Écris le nom du groupe.');
      if (members.length < 2) return $('#msg').innerHTML = getEmojiImg('💡 Il faut au moins 2 membres.');
      if (new Set(members.map(m => m.n.toLowerCase())).size < members.length) {
        return $('#msg').innerHTML = getEmojiImg('💡 Deux membres ont le même prénom.');
      }
      if (S.groups[gn]) return $('#msg').innerHTML = getEmojiImg('💡 Ce nom de groupe existe déjà.');

      const start = +$('#sl').value;
      const newGroup = { members, prog: {}, start };
      applyStartLevel(newGroup, start);

      S.groups[gn] = newGroup;
      S.cur = gn;
      save();
      map();
    };
  }

  /* ---------- Levels Map ---------- */
  function map() {
    const g = G();
    if (!g) return home();
    const allDone = LV.every((_, i) => done(g, i));
    const maxPerMember = LV.length * 3;
    const maxGroup = maxPerMember * g.members.length;

    view(`<div class="card">
      <h1>Groupe « ${esc(S.cur)} »</h1>
      <div class="gs">⭐ Étoiles du groupe : <b>${gStars(g)}</b> / ${maxGroup}</div>
      <div>${g.members.map(m => `<span class="chip">${m.av} ${esc(m.n)} : ⭐ ${mStars(g, m)} / ${maxPerMember}</span>`).join('')}</div>
      <div class="robot" aria-label="Outils retrouvés de Pixo">${LV.map((l, i) => `<span title="${esc(l.title)}">${done(g, i) ? l.part : '❔'}</span>`).join('')}</div>
      ${allDone ? `<h2>🎉 Bravo ! Tout le groupe a terminé tous les niveaux !</h2><button type="button" class="btn" id="bil">🏆 Voir le diplôme du groupe</button>` : ''}
    </div>
    <div class="levels">${LV.map((l, i) => {
      const finishedCount = g.members.filter(m => g.prog && g.prog[i] && g.prog[i][m.n] != null).length;
      const isOk = unlocked(g, i);
      const isDone = done(g, i);
      const earned = lvGroupStars(g, i);
      const maxLv = g.members.length * 3;
      const worldHeader = W[i] ? `<h2 class="world-title">${W[i].icon} ${esc(W[i].title)}</h2>` : '';
      return worldHeader + `<button type="button" class="card lv ${isDone ? 'ok' : ''} ${isOk ? '' : 'lock'}" data-l="${i}" ${isOk ? '' : 'disabled'} aria-label="Niveau ${l.globalNum} : ${esc(l.title)}">
        <div class="lv-icons">
          <span>${isOk ? (isDone ? '✅' : '📋') : '🔒'}</span>
          <span>${l.icon}</span>
        </div>
        <div class="lv-num">Niveau ${l.globalNum}</div>
        <div class="lv-name">${esc(l.title)}</div>
        <div class="lv-stars">⭐ ${earned} / ${maxLv}</div>
        <small>${isDone ? '🔁 Cliquer pour rejouer' : `${finishedCount} / ${g.members.length} élèves`}</small>
      </button>`;
    }).join('')}</div>`);

    if ($('#bil')) $('#bil').onclick = bilan;
    app.querySelectorAll('[data-l]').forEach(btn => {
      btn.onclick = () => {
        const i = +btn.dataset.l;
        if (unlocked(g, i)) openLevel(i);
      };
    });
  }

  /* ---------- Tree HTML Renderer ---------- */
  function withContextNames(str, memberName, groupName) {
    return String(str == null ? '' : str)
      .replace(/\{name\}/g, memberName || 'Élève')
      .replace(/\{group\}/g, groupName || S.cur || 'Groupe');
  }

  function renderTreeBranch(node, memberName, groupName) {
    if (!node) return '';
    const nodeName = withContextNames(node.name, memberName, groupName);
    const hasChildren = Array.isArray(node.children) && node.children.length > 0;
    const isFolder = node.icon === '📂' || node.icon === '📁' || node.icon === '🗑️' || hasChildren;
    const icon = node.icon || (isFolder ? '📁' : iconForKind(node.kind, nodeName));
    const cls = node.create ? 'ftree-node create' : (node.deleted ? 'ftree-node deleted' : 'ftree-node');
    const badge = node.create ? ' <span class="badge-create">➕ À créer</span>' : '';
    const sub = hasChildren ? `<ul>${node.children.map(ch => `<li>${renderTreeBranch(ch, memberName, groupName)}</li>`).join('')}</ul>` : '';
    return `<span class="${cls}">${icon} <b>${esc(nodeName)}</b>${badge}</span>${sub}`;
  }

  function renderExpectedTree(tree, memberName, groupName) {
    if (!tree) return '';
    return `<div class="ftree" aria-label="Résultat attendu">
      <h3>🎯 Résultat attendu</h3>
      <div class="ftree-root">${tree.icon || '📂'} <span>${esc(withContextNames(tree.name, memberName, groupName))}</span></div>
      ${Array.isArray(tree.children) && tree.children.length ? `<ul>${tree.children.map(ch => `<li>${renderTreeBranch(ch, memberName, groupName)}</li>`).join('')}</ul>` : ''}
    </div>`;
  }

  /* ---------- One-time Wipe Warning Dialog ---------- */
  const warnDlg = $('#warn-dlg');
  function confirmWipeIfNeeded(level, onConfirm) {
    if (!level.wipeWarning || S.wipe_confirmed) {
      return onConfirm();
    }
    warnDlg.innerHTML = getEmojiImg(`
      <h2 id="warn-t">⚠️ Attention avant de commencer</h2>
      <p>Pour préparer le <b>Niveau ${level.globalNum} (${esc(level.title)})</b>, Pixo va vider le contenu de :</p>
      <p class="hint">📂 ${esc(level.wipeFoldersText || 'Téléchargements, Images, Musique et Vidéos')}</p>
      <p>Vérifie qu'aucun document personnel important ne s'y trouve.</p>
      <div style="margin-top:16px">
        <button type="button" class="btn" id="warn-ok">✅ J'ai compris, commencer</button>
        <button type="button" class="btn ghost" id="warn-cancel">Fermer</button>
      </div>
    `);
    $('#warn-ok').onclick = () => {
      S.wipe_confirmed = true;
      save();
      warnDlg.close();
      onConfirm();
    };
    $('#warn-cancel').onclick = () => warnDlg.close();
    warnDlg.showModal();
  }

  /* ---------- Level Play & Turn Flow ---------- */
  function openLevel(li) {
    const g = G();
    if (!g) return home();
    const m = nextMember(g, li);
    if (m) {
      return renderTurnScreen(li, m, false);
    }
    return renderLevelGroupSummary(li);
  }

  function renderTurnScreen(li, member, autoStart) {
    const g = G();
    const L = LV[li];
    let started = false;

    const turnOrderHtml = g.members.map(x => {
      const isCur = x.n === member.n;
      const hasPlayed = g.prog && g.prog[li] && g.prog[li][x.n] != null;
      const stCount = star(g, li, x);
      return `<span class="turn-chip ${isCur ? 'active' : ''}">${x.av} ${esc(x.n)} ${hasPlayed ? '⭐'.repeat(stCount || 0) || '✓' : ''}</span>`;
    }).join('');

    const stepsHtml = (L.steps || []).map((st, idx) => `
      <div class="step-card">
        <span class="step-num">${idx + 1}</span>
        <div class="step-text">${esc(withContextNames(st.text, member.n, S.cur))}</div>
      </div>
    `).join('');

    view(`
      <div class="card accent">
        <div class="turn-strip">
          <div class="turn-cur">
            <span class="turn-cur-av">${member.av}</span>
            <div>
              <h2>Niveau ${L.globalNum} : ${L.icon} ${esc(L.title)}</h2>
              <div>Tour de <b>${esc(member.n)}</b></div>
            </div>
          </div>
          <div class="turn-others" aria-label="Ordre de passage">${turnOrderHtml}</div>
        </div>
      </div>

      <div class="card">
        <div class="step-grid">${stepsHtml}</div>
        ${renderExpectedTree(L.tree, member.n, S.cur)}
      </div>

      <div class="card">
        <div style="display:flex;align-items:center;justify-content:space-between;flex-wrap:wrap;gap:10px">
          <div>
            <button type="button" class="btn" id="btn-start">🚀 Commencer</button>
            <button type="button" class="btn" id="btn-verify" disabled>✅ Vérifier</button>
            <button type="button" class="btn ghost s" id="btn-back-map">⬅ Carte</button>
          </div>
          <div>
            <span id="task-state" class="status-pill">En attente de « Commencer »</span>
          </div>
        </div>
        <p class="hint" id="task-hint" role="status"></p>
      </div>
    `);

    $('#btn-back-map').onclick = map;

    const startBtn = $('#btn-start');
    const verifyBtn = $('#btn-verify');
    const statePill = $('#task-state');
    const hintEl = $('#task-hint');

    const doPrepare = async () => {
      startBtn.disabled = true;
      verifyBtn.disabled = true;
      hintEl.textContent = '';
      statePill.className = 'status-pill';
      statePill.textContent = 'Préparation...';
      announce('Préparation des dossiers en cours...');

      const res = await Bridge.startTask(L.id, member.n, S.cur);
      startBtn.disabled = false;
      if (!res || !res.ok) {
        const errMsg = (res && res.error) ? res.error : 'Ferme les fenêtres ouvertes puis réessaie';
        statePill.textContent = 'Non démarré';
        hintEl.innerHTML = getEmojiImg('💡 ' + esc(errMsg));
        announce(errMsg);
        return;
      }

      started = true;
      verifyBtn.disabled = false;
      startBtn.innerHTML = getEmojiImg('🔁 Recommencer à zéro');
      statePill.className = 'status-pill ready';
      statePill.innerHTML = getEmojiImg('🟢 En cours : à toi de jouer !');
      announce('En cours : les fichiers sont prêts.');
    };

    startBtn.onclick = () => confirmWipeIfNeeded(L, doPrepare);

    verifyBtn.onclick = async () => {
      if (!started) return;
      verifyBtn.disabled = true;
      announce('Vérification en cours...');
      const res = await Bridge.verifyTask(L.id, member.n, S.cur);
      renderResultScreen(li, member, res);
    };

    if (autoStart) {
      confirmWipeIfNeeded(L, doPrepare);
    }
  }

  /* ---------- Result View (After Verify) ---------- */
  function renderResultScreen(li, member, res) {
    const g = G();
    const stars = Math.max(0, Math.min(3, +(res && res.stars) || 0));
    const score = +(res && res.score) || 0;
    const checks = (res && Array.isArray(res.checks)) ? res.checks : [];

    g.prog = g.prog || {};
    g.prog[li] = g.prog[li] || {};
    const prev = g.prog[li][member.n];
    const best = prev != null ? Math.max(prev, stars) : stars;
    g.prog[li][member.n] = best;
    save();

    if (stars === 3) {
      try { confetti({ particleCount: 140, spread: 80, origin: { y: 0.6 } }); } catch (e) {}
    }

    const nextM = nextMember(g, li);
    const titleMsg = stars === 3
      ? `Bravo ${esc(member.n)} ! Tout est parfait !`
      : stars >= 1
      ? `Bien joué ${esc(member.n)} ! Tu peux encore améliorer ton score.`
      : `Continue ${esc(member.n)}, regarde les indices ci-dessous !`;

    const checksHtml = checks.map(c => {
      const ok = !!c.ok;
      const chips = Array.isArray(c.items) && c.items.length
        ? `<div class="chk-chips">${c.items.map(it => `<span class="fchip">${iconForKind(it.kind, it.name)} ${esc(it.name)}</span>`).join('')}</div>`
        : '';
      const hintLine = (!ok && c.hint)
        ? `<div class="chk-hint">${esc(c.hint.startsWith('💡') ? c.hint : '💡 ' + c.hint)}</div>`
        : '';
      return `<div class="chk-row ${ok ? 'ok' : 'fail'}">
        <div class="chk-head">
          <span>${ok ? '✅' : '❌'}</span>
          <span>${esc(c.label)}</span>
        </div>
        ${hintLine}
        ${chips}
      </div>`;
    }).join('');

    view(`
      <div class="card" style="text-align:center">
        <h1>${member.av} ${titleMsg}</h1>
        <div class="stars" aria-label="${stars} étoiles sur 3">${'⭐'.repeat(stars)}${'☆'.repeat(3 - stars)}</div>
        <p><b>Score : ${score} %</b>${prev != null && best > stars ? ` · Meilleur score gardé : ${'⭐'.repeat(best)}` : ''}</p>
        ${res && res.note ? `<p><small>${esc(res.note)}</small></p>` : ''}

        <div class="chk-list">${checksHtml}</div>

        <div style="margin-top:18px">
          <button type="button" class="btn ghost" id="btn-replay">🔁 Rejouer</button>
          <button type="button" class="btn" id="btn-next">
            ${nextM ? `➡️ Suivant : ${nextM.av} ${esc(nextM.n)}` : '➡️ Bilan du groupe'}
          </button>
        </div>
      </div>
    `);

    announce(`${member.n} a obtenu ${stars} étoiles (${score} pourcent).`);

    $('#btn-replay').onclick = () => renderTurnScreen(li, member, true);
    $('#btn-next').onclick = () => {
      if (nextM) {
        renderTurnScreen(li, nextM, false);
      } else {
        renderLevelGroupSummary(li);
      }
    };
  }

  /* ---------- Group Summary After Last Member ---------- */
  function renderLevelGroupSummary(li) {
    const g = G();
    const L = LV[li];
    const hasNextLevel = li + 1 < LV.length;

    view(`
      <div class="card accent" style="text-align:center">
        <h1>${L.icon} Niveau ${L.globalNum} : ${esc(L.title)} terminé !</h1>
        <p>Pixo récupère ${esc(L.pn)} ${L.part} !</p>
        <div class="levels" style="margin-top:16px">
          ${g.members.map((m, idx) => {
            const st = star(g, li, m);
            return `<div class="card" style="text-align:center;margin:0">
              <h2>${m.av} ${esc(m.n)}</h2>
              <div class="stars" style="font-size:40px">${'⭐'.repeat(st)}${'☆'.repeat(3 - st)}</div>
              <button type="button" class="btn ghost s" data-replay="${idx}">🔁 Rejouer</button>
            </div>`;
          }).join('')}
        </div>
        <div style="margin-top:18px">
          ${hasNextLevel ? `<button type="button" class="btn" id="btn-next-lv">➡️ Niveau suivant (Niveau ${LV[li + 1].globalNum})</button>` : `<button type="button" class="btn" id="btn-bilan">🏆 Voir le diplôme du groupe</button>`}
          <button type="button" class="btn ghost" id="btn-map">🏠 Retour à la carte</button>
        </div>
      </div>
    `);

    app.querySelectorAll('[data-replay]').forEach(b => {
      b.onclick = () => renderTurnScreen(li, g.members[+b.dataset.replay], false);
    });
    if ($('#btn-next-lv')) $('#btn-next-lv').onclick = () => openLevel(li + 1);
    if ($('#btn-bilan')) $('#btn-bilan').onclick = bilan;
    $('#btn-map').onclick = map;
  }

  /* ---------- Final Group Diploma (Bilan) ---------- */
  const rank = p => p >= 0.9
    ? ['🏆', 'Maître des dossiers']
    : p >= 0.7
    ? ['🥇', 'Expert du rangement']
    : p >= 0.5
    ? ['🥈', 'Explorateur organisé']
    : ['🥉', 'Apprenti courageux'];

  function bilan() {
    const g = G();
    if (!g) return home();
    const mx = LV.length * 3;
    const tot = gStars(g);
    const gp = mx * g.members.length > 0 ? tot / (mx * g.members.length) : 0;
    const r = rank(gp);

    const cards = g.members.map(m => {
      const t = mStars(g, m);
      const mr = rank(mx > 0 ? t / mx : 0);
      return `<div class="card" style="text-align:center">
        <h2>${m.av} ${esc(m.n)}</h2>
        <div class="stars">${mr[0]}</div>
        <h3>${mr[1]}</h3>
        <p>⭐ ${t} sur ${mx} étoiles</p>
      </div>`;
    }).join('');

    view(`
      <div class="card accent" style="text-align:center">
        <h1>🎓 Diplôme du groupe « ${esc(S.cur)} »</h1>
        <div class="stars">${r[0]}</div>
        <h2>${r[1]}</h2>
        <p>Score du groupe : ⭐ <b>${tot}</b> sur ${mx * g.members.length} étoiles (${Math.round(gp * 100)} %)</p>
        <div>
          <span class="chip on">🤝 Esprit d'équipe</span>
          <span class="chip on">📁 As des dossiers</span>
        </div>
      </div>
      <div class="levels">${cards}</div>
      <div>
        <button type="button" class="btn" id="pr">🖨️ Imprimer le diplôme</button>
        <button type="button" class="btn ghost" id="bk">⬅ Retour à la carte</button>
      </div>
    `);
    $('#pr').onclick = () => window.print();
    $('#bk').onclick = map;
    try { confetti({ particleCount: 200, spread: 100, origin: { y: 0.5 } }); } catch (e) {}
  }

  /* ---------- Group Management Dialog (Ctrl + Shift + G) ---------- */
  const adm = $('#adm');
  let admKeys = null;
  let admLeft = false;

  function admRender() {
    const names = Object.keys(S.groups);
    adm.innerHTML = getEmojiImg(
      `<h2 id="adm-t">👥 Gérer les groupes</h2>` +
      (names.length ? names.map(n => {
        const g = S.groups[n];
        const doneCount = LV.filter((_, i) => done(g, i)).length;
        const curStart = g.start || 0;
        const isCurrent = S.cur === n;
        return `<div class="grow">
          <div style="flex:1;min-width:220px">
            <b>${esc(n)}</b> ${isCurrent ? '<span class="chip on" style="padding:2px 10px;font-size:15px">Actif</span>' : ''}<br>
            <small>${g.members.map(m => `${m.av} ${esc(m.n)}`).join(', ')} · ${doneCount}/${LV.length} niveaux réussis</small>
            <label style="margin-top:6px;font-size:16px">Niveau de départ :
              <select data-start="${esc(n)}">
                ${LV.map((l, idx) => `<option value="${idx}" ${idx === curStart ? 'selected' : ''}>Niveau ${l.globalNum} : ${esc(l.title)}</option>`).join('')}
              </select>
            </label>
          </div>
          <div class="grow-actions">
            ${!isCurrent ? `<button type="button" class="btn s" data-pick="${esc(n)}">Choisir</button>` : ''}
            <button type="button" class="btn danger" data-del="${esc(n)}">🗑️ Supprimer</button>
          </div>
        </div>`;
      }).join('') : '<p>Aucun groupe pour le moment.</p>') +
      `<div style="margin-top:14px">
        <button type="button" class="btn" id="adm-new">➕ Créer un groupe</button>
        <button type="button" class="btn ghost" id="adm-x">Fermer</button>
      </div>
      <p><small>Raccourci clavier : <span class="kb">Ctrl</span> + <span class="kb">Maj</span> + <span class="kb">G</span></small></p>`
    );

    adm.querySelectorAll('[data-pick]').forEach(b => {
      b.onclick = () => {
        S.cur = b.dataset.pick;
        admLeft = true;
        save();
        adm.close();
        map();
      };
    });

    adm.querySelectorAll('[data-start]').forEach(sel => {
      sel.onchange = () => {
        const grpName = sel.dataset.start;
        const g = S.groups[grpName];
        if (!g) return;
        applyStartLevel(g, +sel.value);
        save();
        admRender();
        if (S.cur === grpName) map();
      };
    });

    adm.querySelectorAll('[data-del]').forEach(b => {
      b.onclick = () => {
        if (!b.dataset.arm) {
          b.dataset.arm = '1';
          b.innerHTML = getEmojiImg('⚠️ Sûr ? Clique encore');
          return;
        }
        const n = b.dataset.del;
        delete S.groups[n];
        if (S.cur === n) {
          const rem = Object.keys(S.groups);
          S.cur = rem.length ? rem[0] : null;
        }
        admLeft = true;
        save();
        admRender();
        home();
      };
    });

    $('#adm-new').onclick = () => {
      admLeft = true;
      adm.close();
      setup(false);
    };
    $('#adm-x').onclick = () => adm.close();
  }

  function admOpen() {
    if (adm.open) return adm.close();
    admKeys = [window.onkeydown, window.onkeyup];
    admLeft = false;
    window.onkeydown = window.onkeyup = null;
    admRender();
    adm.showModal();
  }

  adm.addEventListener('close', () => {
    if (!admLeft && admKeys) [window.onkeydown, window.onkeyup] = admKeys;
    admKeys = null;
  });

  /* Disable browser right-click context menu throughout the app */
  document.addEventListener('contextmenu', e => e.preventDefault());

  document.addEventListener('keydown', e => {
    if (e.ctrlKey && e.shiftKey && !e.altKey && !e.metaKey && e.code === 'KeyG') {
      e.preventDefault();
      admOpen();
    }
  });

  /* ---------- Header Initialization & Startup ---------- */
  $('#hdr-logo').innerHTML = getEmojiImg($('#hdr-logo').textContent);
  $('#bHome').innerHTML = getEmojiImg($('#bHome').textContent);
  $('#bHome').onclick = home;

  async function init() {
    const loaded = await Bridge.loadState();
    if (loaded && loaded.v === 1) {
      S = loaded;
    }
    const env = await Bridge.checkEnv();
    if (env && env.banner) {
      bannerEl.hidden = false;
      bannerEl.innerHTML = getEmojiImg('⚠️ ' + esc(env.banner));
    } else {
      bannerEl.hidden = true;
    }
    home();
  }

  init();
})();
