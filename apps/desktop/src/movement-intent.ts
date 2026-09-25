import type { MovementMode, MovementOptions, MoveStep, Point } from './tactical-api';

const directions:Record<string,Point>={north:{x:0,y:-1,z:0},south:{x:0,y:1,z:0},east:{x:1,y:0,z:0},west:{x:-1,y:0,z:0},northeast:{x:1,y:-1,z:0},northwest:{x:-1,y:-1,z:0},southeast:{x:1,y:1,z:0},southwest:{x:-1,y:1,z:0},up:{x:0,y:0,z:1},down:{x:0,y:0,z:-1}};
export type MovementProposal={path:MoveStep[];error:null}|{path:null;error:string};

// A narrow local proposal parser. It never queries hidden geometry or grants movement.
export function proposeMovement(text:string,options:MovementOptions):MovementProposal {
  const fail=(error:string):MovementProposal=>({path:null,error});
  if(!text.trim())return fail('Describe your route using a distance and direction.');
  if(text.length>512)return fail('Describe a shorter route.');
  const legs=text.trim().replace(/\.$/,'').split(/\s*(?:,\s*(?:then\s+)?|\bthen\b)\s*/i);
  if(legs.length>32)return fail('Use at most 32 parts in one route.');
  let position={...options.position};let mode:MovementMode='Walk';const path:MoveStep[]=[];
  for(const leg of legs){
    const match=/^(?:(move|walk|crawl|climb|swim|fly|burrow|jump)\s+)?(\d+(?:\.\d+)?)\s*(?:feet|foot|ft)\s+(north(?:[ -]?east|[ -]?west)?|south(?:[ -]?east|[ -]?west)?|east|west|up|down)$/i.exec(leg);
    if(!match)return fail('Use a route such as “walk 10 feet north, then 5 feet east”. Other actions need their own choice.');
    if(match[1]&&match[1].toLowerCase()!=='move')mode=(match[1][0].toUpperCase()+match[1].slice(1).toLowerCase()) as MovementMode;
    if(!options.modes.includes(mode))return fail(`${mode} is not available for this creature.`);
    const units=Number(match[2])*2;const direction=directions[match[3].toLowerCase().replace(/[ -]/g,'')];
    const increment=direction.z===0?options.grid_units:1;
    if(!Number.isSafeInteger(units)||units<=0||units%increment!==0)return fail(`Use positive distances in ${increment/2}-foot increments for this direction.`);
    let remaining=units;
    while(remaining>0){
      if(path.length>=128)return fail('Use a shorter route; you can continue moving afterward.');
      const step=Math.min(10,remaining);
      position={x:position.x+direction.x*step,y:position.y+direction.y*step,z:position.z+direction.z*step};
      path.push({destination:position,mode});remaining-=step;
    }
  }
  return {path,error:null};
}
