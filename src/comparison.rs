// timer comparison types
pub enum Comparison {
	PersonalBest,
	Golds,
	None
}

impl Comparison {
	pub fn next(&mut self) {
		match self {
			Comparison::PersonalBest => {
				*self = Comparison::Golds;
    			},
    			Comparison::Golds => {
				*self = Comparison::None;
        		},
        		Comparison::None => {
				*self = Comparison::PersonalBest;
            		}
    		} 
    	}
    	pub fn prev(&mut self) {
		match self {
			Comparison::PersonalBest => {
				*self = Comparison::None;
    			},
    			Comparison::Golds => {
				*self = Comparison::PersonalBest;
        		}
        		Comparison::None => {
				*self = Comparison::Golds;
            		}
    		}
        }
}
