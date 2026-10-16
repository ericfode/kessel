'use strict';

function deployLeaderSystem(kind,obj,defaultTarget,api){
 if(kind!=='deploy')return null;
 for(const[k,v]of obj.props)if(k!=='target')api.fail(obj.ctx,v,'deploy-leader-field:'+k);
 return {kind:'deployLeaderSystem',target:api.target(obj.props.get('target'),obj.ctx,defaultTarget),targetTypes:['leader'],event:'onLeaderDeployed',eligibility:{kind:'binary',op:'&&',left:{kind:'cardPredicate',name:'isLeader'},right:{kind:'unary',op:'!',value:{kind:'binary',op:'&&',left:{kind:'cardPredicate',name:'isDeployableLeader'},right:{kind:'cardProperty',name:'deployed'}}}},inheritedEligibility:'CardTargetSystem.canAffectInternal',handler:[{kind:'assertCardPredicate',name:'isDeployableLeader',expected:true,failure:'throw'},{kind:'invokeClassContract',method:'deploy',args:[{type:'leaderUnit'}]}],contingentEvents:[{event:'onUnitEntersPlay',player:'contextPlayer',card:'deployedCard'}],contingentGeneratorRegistration:'updateEventAfterSuper',defaultProperties:'pinnedCardTargetSystem'};
}
module.exports={deployLeaderSystem};
