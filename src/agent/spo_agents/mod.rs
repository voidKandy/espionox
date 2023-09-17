#[cfg(feature = "meal_planner")]
pub mod meal_planner;
pub mod summarizer;

pub use summarizer::SummarizerAgent;

pub trait SpecialAgent {
    fn init(env: ConfigEnv) -> Self {
        MealPlannerAgent(
            Agent::build(MealPlannerAgent::get_ingredients_settings(), env)
                .expect("Failed to initialize summarizer agent"),
        )
    }
}
