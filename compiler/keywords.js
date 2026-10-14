'use strict';
// Printed-keyword parser shared by the compiler, the card pipeline planners and their tests.
// Extracted from swu_bot's bot/omni/rs/make-pool.js (whose history this file keeps); the
// strict-pool generator itself drives the forceteki engine and stays in swu_bot.
const ALLOWED = new Set(['sentinel', 'raid', 'overwhelm', 'saboteur', 'restore', 'ambush', 'grit', 'shielded', 'hidden']);

/** Whitelist the whole text, not just its first keyword or the data's keyword tags. */
function parseKeywords(c) {
    if (c.pilotText || (c.keywords ?? []).some((k) => !ALLOWED.has(k))) return null;
    let reminderDepth=0;
    for (const char of c.text ?? '') {
        if (char==='(') {if (reminderDepth!==0) return null;reminderDepth=1;}
        else if (char===')') {if (reminderDepth!==1) return null;reminderDepth=0;}
    }
    if (reminderDepth!==0) return null;
    const text = (c.text ?? '').replace(/\([^)]*\)/g, '').trim();
    if (!text) return {};
    const out = {};
    for (const line of text.split(/\n|,/).map((s) => s.trim()).filter(Boolean)) {
        const m = /^(Sentinel|Raid|Overwhelm|Saboteur|Restore|Ambush|Grit|Shielded|Hidden)(?:\s+(\d+))?\.?$/i.exec(line);
        if (!m) return null;
        const k = m[1].toLowerCase();
        if ((k === 'raid' || k === 'restore') !== (m[2] !== undefined)) return null;
        if (Object.hasOwn(out,k)) return null;
        const amount=m[2] ? Number(m[2]) : 1;
        if (!Number.isSafeInteger(amount) || amount > 65535) return null;
        out[k] = amount;
    }
    return out;
}
module.exports = { parseKeywords };
