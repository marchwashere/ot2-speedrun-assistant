//! Skill purchases funded by the JP100 victory, without borrowing later boss JP.
use crate::{Result,ensure,world::WorldState};
use serde::Serialize;

// JobData.JPCost. Ordinary jobs start with two free commands. Arcanist's
// four planned additional commands cost 2,000 apiece. EX/Inventor cost no JP.
const STANDARD:[u32;8]=[0,0,30,100,500,1000,3000,5000];
const ARCANIST:[u32;8]=[0,0,2000,2000,2000,2000,2000,5000];
const PLANS:&[(u32,&str,&[(u32,usize)])]=&[
    (7,"Throné",&[(1,8),(2,7)]), // All Thief commands; Deal More Damage.
    (6,"Osvald",&[(6,5),(0,5),(2,7),(9,6)]), // Includes later Price of Power.
    (4,"Partitio",&[(0,5),(5,4),(2,6),(4,8)]),
    (5,"Temenos",&[(5,4),(4,8)]),
];

#[derive(Debug,Serialize)]
pub struct Budget {
    pub character_id:u32,pub name:&'static str,pub available:u32,
    pub remaining_skill_cost:u32,pub shortfall:u32,
}

pub fn audit(w:&WorldState)->Result<Vec<Budget>> {
    PLANS.iter().map(|&(id,name,jobs)| {
        let p=w.character_progression.get(&id)
            .ok_or_else(||crate::SimError(format!("Missing JP ledger for {name}")))?;
        ensure(p.jp_spent<=p.jp_earned,format!("Invalid JP ledger for {name}"))?;
        let remaining_skill_cost:u32=jobs.iter().map(|&(job,target)| {
            let learned=p.jobs.learned_abilities.get(&job).map_or(2,|a|a.len().max(2));
            let costs=if job==9 {&ARCANIST}else{&STANDARD};
            costs[learned.min(target)..target].iter().sum::<u32>()
        }).sum();
        let available=p.jp_earned-p.jp_spent;
        Ok(Budget{character_id:id,name,available,remaining_skill_cost,
            shortfall:remaining_skill_cost.saturating_sub(available)})
    }).collect()
}

pub fn sufficient(w:&WorldState)->Result<bool> {
    Ok(audit(w)?.iter().all(|b|b.shortfall==0))
}

/// Cheap preflight for the fixed Roque reward, including only active recipients.
pub(crate) fn can_fund_with(w:&WorldState,jp:u32)->Result<bool> {
    Ok(audit(w)?.iter().all(|b|b.shortfall<=if w.party.contains(&b.character_id){jp}else{0}))
}

#[cfg(test)]mod tests {
    use super::*;
    fn fixture()->WorldState {
        let v:serde_json::Value=serde_json::from_str(include_str!("../data/source/galdera-flex-entry.json")).unwrap();
        serde_json::from_value(v["world_after"].clone()).unwrap()
    }
    #[test]fn skill_totals_and_already_purchased_commands() {
        let mut w=fixture();
        for &(id,_,_) in PLANS {
            let p=w.character_progression.get_mut(&id).unwrap();
            p.jobs.learned_abilities.clear();p.jp_earned=0;p.jp_spent=0;
        }
        let budgets=audit(&w).unwrap();
        assert_eq!(budgets.iter().map(|b|b.remaining_skill_cost).collect::<Vec<_>>(),[14260,13890,12020,9760]);
        for b in budgets {w.character_progression.get_mut(&b.character_id).unwrap().jp_earned=b.remaining_skill_cost;}
        assert!(sufficient(&w).unwrap());
        crate::gravell_setup::learn(&mut w,6,6,&["ABI_SET_SCH_030","ABI_SET_SCH_060","ABI_SET_SCH_040"]).unwrap();
        assert!(sufficient(&w).unwrap());
        // An unrelated expense must not be hidden by a lifetime-earned threshold.
        w.character_progression.get_mut(&6).unwrap().jp_spent+=30;
        assert_eq!(audit(&w).unwrap()[1].shortfall,30);
        assert!(!sufficient(&w).unwrap());
    }
    #[test]fn reported_preemptive_reward_cannot_fund_aebers() {
        let mut w=fixture();
        let p=w.character_progression.get_mut(&7).unwrap();
        p.jobs.learned_abilities.clear();p.jp_earned=2399;p.jp_spent=0;
        // Other recipients are funded so this isolates Throné's deadline.
        for id in [6,4,5] {w.character_progression.get_mut(&id).unwrap().jp_earned=30000;}
        assert!(!can_fund_with(&w,11100).unwrap());
        w.character_progression.get_mut(&7).unwrap().jp_earned+=11100;
        assert_eq!(audit(&w).unwrap()[0].shortfall,761);
        w.character_progression.get_mut(&7).unwrap().jp_earned+=45;
        assert_eq!(audit(&w).unwrap()[0].shortfall,716);
        assert!(can_fund_with(&w,716).unwrap());
        assert!(!can_fund_with(&w,715).unwrap());
    }
}
