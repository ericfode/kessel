'use strict';

// Bounty is a keyword-owned registration, not an ordinary death trigger. Preserve
// its collection protocol explicitly for the later bulk engine implementation.
// The Rust Ability schema deliberately rejects this metadata until then.
function bountyAbility(input, ctx, {ability, fail}) {
    const result = ability(input, ctx, 'bounty');
    if (result.optional !== undefined && typeof result.optional !== 'boolean') {
        fail(ctx, input, 'bounty-optional-boolean');
    }
    result.title = `Collect Bounty: ${result.title}`;
    result.optional ??= true;
    result.bounty = {
        events: ['onCardDefeated', 'onCardCaptured'],
        eventCard: 'source',
        canBeTriggeredBy: 'opponent',
        contextPlayer: 'collector',
        zone: 'anyArena',
        canResolveWithoutLegalTargets: true,
        emitsCollectedEvent: 'onBountyCollected',
    };
    return result;
}

function bountyGrant(obj, duration, target, {ability, literal, fail}) {
    for(const [key,value] of obj.props) {
        if(!['keyword','ability'].includes(key))fail(obj.ctx,value,'unsupported-bounty-grant-field:'+key);
    }
    if(!obj.props.has('keyword')||literal(obj.props.get('keyword'),obj.ctx)!=='bounty'||!obj.props.has('ability')) {
        fail(obj.ctx,obj.node,'bounty-grant-keyword-and-ability');
    }
    return {kind:'gainBounty',target,duration,abilitySource:'recipient',
        ability:bountyAbility(obj.props.get('ability'),obj.ctx,{ability,fail})};
}

module.exports = {bountyAbility,bountyGrant};
