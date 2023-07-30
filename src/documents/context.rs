use super::{
    factory::{Factory, Initializer},
    Document,
};
use crate::utils::{load::load_json, schema::discover_schema_id};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use url::Url;

#[derive(Default)]
pub struct Context {
    /**
     * keep track of retrieved documents
     */
    retrieved: HashSet<Url>,
    /**
     * document factories by schema identifier
     */
    factories: HashMap<Url, Factory>,
    /**
     * all documents, indexed by document id
     */
    documents: HashMap<Url, Box<dyn Document>>,
    /**
     * maps node urls to their documents
     */
    node_documents: HashMap<Url, Url>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_factory(&mut self, schema_url: Url, factory: Factory) {
        self.factories.insert(schema_url, factory);
    }

    pub fn load_from_url(
        &mut self,
        retrieval_url: &Url,
        given_url: &Url,
        antecedent_url: Option<&Url>,
        default_schema_id: &str,
    ) -> Result<(), &'static str> {
        if self.retrieved.contains(retrieval_url) {
            return Ok(());
        }
        self.retrieved.insert(retrieval_url.clone());

        let document_node = load_json(retrieval_url)?;
        self.load_from_node(
            retrieval_url,
            given_url,
            antecedent_url,
            document_node,
            default_schema_id,
        )?;

        Ok(())
    }

    pub fn load_from_node(
        &mut self,
        retrieval_url: &Url,
        given_url: &Url,
        antecedent_url: Option<&Url>,
        document_node: Value,
        default_schema_id: &str,
    ) -> Result<(), &'static str> {
        let schema_id = discover_schema_id(&document_node).unwrap_or(default_schema_id);
        let factory = self
            .factories
            .get(&schema_id.parse().unwrap())
            .ok_or("factory not found")?;
        let document = factory(Initializer {
            retrieval_url,
            given_url,
            antecedent_url,
            document_node,
        });
        let document_url = document.document_url();

        for node_url in document.get_node_urls() {
            if self
                .node_documents
                .insert(node_url.clone(), document_url.clone())
                .is_some()
            {
                return Err("duplicate node");
            }
        }

        if self
            .documents
            .insert(document_url.clone(), document)
            .is_some()
        {
            return Err("duplicate document");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents;
    use crate::schemas;
    use std::env::current_dir;

    #[test]
    fn t() {
        let mut context = Context::new();
        context.register_factory(
            schemas::intermediate_a::SCHEMA_ID.parse().unwrap(),
            Box::new(
                |Initializer {
                     given_url,
                     document_node,
                     ..
                 }| {
                    Box::new(documents::intermediate_a::Document::new(
                        given_url.clone(),
                        document_node,
                    ))
                },
            ),
        );

        let mut path = current_dir().unwrap();
        path.push("fixtures");
        path.push("testing");
        path.push("schema");
        path.push("jns42-intermediate-a");
        path.push("string-or-boolean.json");

        let url: Url = format!("file://{}", path.to_str().unwrap())
            .parse()
            .unwrap();

        context
            .load_from_url(&url, &url, None, schemas::intermediate_a::SCHEMA_ID)
            .unwrap();
    }
}