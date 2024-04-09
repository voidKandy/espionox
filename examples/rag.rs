use espionox::{
    agents::{
        independent::IndependentAgent,
        memory::{embeddings::EmbeddingVector, messages::MessageRole, Message, ToMessage},
        Agent, AgentError,
    },
    environment::{
        dispatch::{
            listeners::ListenerMethodReturn, Dispatch, EnvListener, EnvMessage, EnvRequest,
        },
        Environment,
    },
    language_models::{openai::embeddings::OpenAiEmbeddingModel, ModelProvider, LLM},
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct RagListener<'p> {
    agent_id: String,
    /// RAG listeners can be used as long as they have some connection to a data source. In this
    /// example we use a vector, but it could be anything, including a Database pool.
    data: Option<DbStruct<'p>>,
    /// It depends on your implementation and Data source, but in this example, our RAG listener
    /// will require access to an embedderr    
    embedder: IndependentAgent,
}

#[derive(Debug, Clone)]
pub struct Product<'p> {
    name: &'p str,
    description: &'p str,
    desc_embedding: EmbeddingVector,
}

#[derive(Debug)]
pub struct DbStruct<'p>(Vec<Product<'p>>);

impl<'p> RagListener<'p> {
    async fn embed(&self, str: &str) -> Result<Vec<f32>, AgentError> {
        Ok(self.embedder.get_embedding(str).await?)
    }

    async fn init_products(&mut self) {
        let data = DbStruct(vec![
        Product {
            name: "SmartWatch 2000",
            description: "Stay connected and track your fitness with the SmartWatch 2000. This sleek device features a vibrant touchscreen, heart rate monitoring, and a variety of smart notifications.",
            desc_embedding: EmbeddingVector::from(self.embed("Stay connected and track your fitness with the SmartWatch 2000. This sleek device features a vibrant touchscreen, heart rate monitoring, and a variety of smart notifications.").await.unwrap()),
        },

        Product {
            name: "Quantum Laptop Pro",
            description: "Unleash the power of productivity with the Quantum Laptop Pro. Equipped with a high-performance processor, stunning display, and a lightweight design, it's your perfect companion for work and play.",
            desc_embedding: EmbeddingVector::from(self.embed("Unleash the power of productivity with the Quantum Laptop Pro. Equipped with a high-performance processor, stunning display, and a lightweight design, it's your perfect companion for work and play.").await.unwrap()),
        },

        Product {
            name: "ZenAir Noise-Canceling Headphones",
            description: "Immerse yourself in crystal-clear sound with the ZenAir Noise-Canceling Headphones. These wireless over-ear headphones offer premium comfort and cutting-edge noise-canceling technology for an unparalleled audio experience.",
            desc_embedding: EmbeddingVector::from(self.embed("Immerse yourself in crystal-clear sound with the ZenAir Noise-Canceling Headphones. These wireless over-ear headphones offer premium comfort and cutting-edge noise-canceling technology for an unparalleled audio experience.").await.unwrap()),
        },

        Product {
            name: "Eco-Friendly Bamboo Water Bottle",
            description: "Make a statement while staying eco-friendly with our Bamboo Water Bottle. Crafted from sustainable bamboo, this stylish and reusable bottle is perfect for staying hydrated on the go.",
            desc_embedding: EmbeddingVector::from(self.embed("Make a statement while staying eco-friendly with our Bamboo Water Bottle. Crafted from sustainable bamboo, this stylish and reusable bottle is perfect for staying hydrated on the go.").await.unwrap()),
        },

        Product {
            name: "Stellar Telescope 4000X",
            description: "Explore the wonders of the night sky with the Stellar Telescope 4000X. This high-powered telescope is perfect for astronomy enthusiasts, featuring advanced optics and a sturdy mount for clear and detailed views.",
            desc_embedding: EmbeddingVector::from(self.embed("Explore the wonders of the night sky with the Stellar Telescope 4000X. This high-powered telescope is perfect for astronomy enthusiasts, featuring advanced optics and a sturdy mount for clear and detailed views.").await.unwrap()),
        },

        Product {
            name: "Gourmet Coffee Sampler Pack",
            description: "Indulge your taste buds with our Gourmet Coffee Sampler Pack. This curated collection includes a variety of premium coffee blends from around the world, offering a delightful coffee experience.",
            desc_embedding: EmbeddingVector::from(self.embed("Indulge your taste buds with our Gourmet Coffee Sampler Pack. This curated collection includes a variety of premium coffee blends from around the world, offering a delightful coffee experience.").await.unwrap()),
        },

        Product {
            name: "Fitness Tracker Pro",
            description: "Achieve your fitness goals with the Fitness Tracker Pro. Monitor your steps, heart rate, and sleep patterns while receiving real-time notifications. Sleek design and long battery life make it an essential companion for an active lifestyle.",
            desc_embedding: EmbeddingVector::from(self.embed("Achieve your fitness goals with the Fitness Tracker Pro. Monitor your steps, heart rate, and sleep patterns while receiving real-time notifications. Sleek design and long battery life make it an essential companion for an active lifestyle.").await.unwrap()),
        },

        Product {
            name: "Retro Arcade Gaming Console",
            description: "Relive the nostalgia of classic arcade games with our Retro Arcade Gaming Console. Packed with your favorite titles, this compact console brings back the joy of retro gaming in a modern and portable design.",
            desc_embedding: EmbeddingVector::from(self.embed("Relive the nostalgia of classic arcade games with our Retro Arcade Gaming Console. Packed with your favorite titles, this compact console brings back the joy of retro gaming in a modern and portable design.").await.unwrap()),
        },

        Product {
            name: "Luxe Leather Messenger Bag",
            description: "Elevate your style with the Luxe Leather Messenger Bag. Crafted from premium leather, this sophisticated bag combines fashion and functionality, offering ample space for your essentials in a timeless design.",
            desc_embedding: EmbeddingVector::from(self.embed("Elevate your style with the Luxe Leather Messenger Bag. Crafted from premium leather, this sophisticated bag combines fashion and functionality, offering ample space for your essentials in a timeless design.").await.unwrap()),
        },

       Product {
            name: "Herbal Infusion Tea Set",
            description: "Unwind and savor the soothing flavors of our Herbal Infusion Tea Set. This carefully curated collection features a blend of herbal teas, each with unique health benefits and delightful aromas.",
            desc_embedding: EmbeddingVector::from(self.embed("Unwind and savor the soothing flavors of our Herbal Infusion Tea Set. This carefully curated collection features a blend of herbal teas, each with unique health benefits and delightful aromas.").await.unwrap()),
        }
    ]);
        self.data = Some(data);
    }
}

/// We'll implement ToMessage for our DbStruct so we have control over how the model sees the data
/// it's given
impl<'p> ToMessage for DbStruct<'p> {
    fn to_message(&self, role: MessageRole) -> Message {
        let mut content = String::from("Answer the user's query based on the provided data:");
        self.0.iter().for_each(|p| {
            content.push_str(&format!(
                "\nProduct Name: {}\nProduct Description: {}",
                p.name, p.description
            ));
        });
        Message { role, content }
    }
}

impl<'p> DbStruct<'p> {
    /// A simple helper function to get similar data given a query embedding
    /// KEEP IN MIND THIS IS JUST FOR AN EXAMPLE, I DO NOT RECCOMEND VECTOR QUERYING AN ARRAY LIKE
    /// THIS IN PROD
    async fn get_close_embeddings_from_query_embedding(
        &self,
        qembed: EmbeddingVector,
        amt: usize,
    ) -> DbStruct<'p> {
        let mut map = HashMap::new();
        let mut scores: Vec<f32> = self
            .0
            .iter()
            .map(|p| {
                let score = qembed.score_l2(&p.desc_embedding);
                map.insert((score * 100.0) as u32, p);
                println!("Score for: {} is {}", p.name, score);
                score
            })
            .collect();
        scores.sort_by(|a, b| a.total_cmp(b));
        let closest = scores[..amt].into_iter().fold(vec![], |mut acc, s| {
            let score_key = (s * 100.0) as u32;
            if let Some(val) = map.remove(&score_key) {
                acc.push(val.to_owned())
            }
            acc
        });
        DbStruct(closest)
    }
}

impl<'p: 'static> EnvListener for RagListener<'p> {
    fn trigger<'l>(&self, env_message: &'l EnvMessage) -> Option<&'l EnvMessage> {
        // We'll have our RAG trigger everytime a completion is requested. This could also be applied
        // to `EnvRequest::GetCompletionStreamHandle`
        if let EnvMessage::Request(req) = env_message {
            if let EnvRequest::GetCompletion { agent_id, .. } = req {
                if agent_id == &self.agent_id {
                    return Some(env_message);
                }
            }
        }
        None
    }

    fn method<'l>(
        &'l mut self,
        trigger_message: EnvMessage,
        dispatch: &'l mut Dispatch,
    ) -> ListenerMethodReturn {
        Box::pin(async move {
            // Once the listener is triggered we'll get mutable access to the watched agent
            let agent = dispatch.get_agent_mut(&self.agent_id).unwrap();
            // We'll grab the last user message sent, so we can have something to embed
            if let Some(latest_user_message) = agent
                .cache
                .as_ref()
                .iter()
                .filter(|m| m.role == MessageRole::User)
                .last()
            {
                let qembed =
                    EmbeddingVector::from(self.embed(&latest_user_message.content).await.unwrap());
                let strcts = self
                    .data
                    .as_ref()
                    .unwrap()
                    .get_close_embeddings_from_query_embedding(qembed, 5)
                    .await;
                println!(
                    "STRUCTS PUSHING: {:?}",
                    strcts.0.iter().map(|p| p.name).collect::<Vec<&str>>()
                );
                // We use the embedding to push relevant structs to our agent's memory
                agent.cache.push(strcts.to_message(MessageRole::System));
            }
            // Return the trigger message unchanged
            Ok(trigger_message)
        })
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let api_key = std::env::var("OPENAI_KEY").unwrap();
    let mut map = HashMap::new();
    map.insert(ModelProvider::OpenAi, api_key);
    let mut env = Environment::new(Some("testing"), map);

    let embedder = Agent::new(
        None,
        LLM::new_embedding_model(OpenAiEmbeddingModel::Small.into(), None),
    );
    let embedder = env
        .make_agent_independent(embedder)
        .await
        .expect("Couldn't make indi agent");

    let agent = Agent::new(Some("You are jerry!!"), LLM::default_openai());
    let mut handle = env.insert_agent(None, agent).await.unwrap();
    let mut listener = RagListener {
        embedder,
        agent_id: handle.id.clone(),
        data: None,
    };

    listener.init_products().await;
    let _ = env.insert_listener(listener).await;

    let mut env_handle = env.spawn_handle().unwrap();

    let ticket = handle
        .request_io_completion(Message::new_user(
            "I need a new fitness toy, what is the best product for me?",
        ))
        .await
        .unwrap();

    let mut stack = env_handle.finish_current_job().await.unwrap();
    let noti = stack.take_by_ticket(ticket).unwrap();
    println!("{:?}", noti);
}
