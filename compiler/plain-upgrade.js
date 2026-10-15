'use strict';

// CardHelpers.createUnimplementedCard constructs an ordinary UpgradeCard for
// BasicUpgrade metadata. Its setupCardAbilities is empty: only its printed
// attachment stats remain. Call this only in build.js's no-source fallback,
// and retain baseClass: 'UpgradeCard' so runtime admission checks the same type.
const ALLOWED_FIELDS = new Set([
    'id', 'internalName', 'title', 'subtitle', 'cost', 'hp', 'power',
    'text', 'deployBox', 'epicAction', 'unique', 'rules',
    'upgradePower', 'upgradeHp', 'aspects', 'traits', 'keywords', 'types',
    'setId', 'setCodes', 'arena'
]);
const ASPECTS = new Set(['vigilance', 'command', 'aggression', 'cunning', 'heroism', 'villainy']);
const empty = value => value == null || value === '';
const boundedStat = value => Number.isSafeInteger(value) && value >= -1000 && value <= 1000;
const strings = value => Array.isArray(value) && value.every(item => typeof item === 'string' && item.length > 0);

function isPlainUpgrade(card) {
    if (!card || typeof card !== 'object' || Array.isArray(card)) return false;
    // Unknown metadata may encode another mechanic; new fields require review.
    if (Object.keys(card).some(key => !ALLOWED_FIELDS.has(key))) return false;
    if (!Array.isArray(card.types) || card.types.length !== 1 || card.types[0] !== 'upgrade') return false;
    if (card.unique !== false || !Array.isArray(card.keywords) || card.keywords.length !== 0) return false;
    if (!['text', 'rules', 'deployBox', 'epicAction'].every(key => empty(card[key])) || card.arena != null) return false;
    if (!['id', 'internalName', 'title'].every(key => typeof card[key] === 'string' && card[key].trim().length > 0)) return false;
    if (!empty(card.subtitle) || !strings(card.aspects) || !card.aspects.every(value => ASPECTS.has(value)) || !strings(card.traits)) return false;
    if (!Number.isSafeInteger(card.cost) || card.cost < 0 || card.cost > 1000) return false;
    if (!boundedStat(card.power) || !boundedStat(card.hp)) return false;
    return card.upgradePower === card.power && card.upgradeHp === card.hp;
}

module.exports = { isPlainUpgrade };
