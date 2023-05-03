#![allow(dead_code, unused_variables, unused_imports, unreachable_code, non_snake_case)]

use crate::input::InputCart as Cart;
use crate::input::InputCartAttribute as Attribute;
use crate::input::InputCartLinesMerchandise::ProductVariant;
use crate::input::InputCartLinesMerchandiseOnProductVariant;
use crate::output::CartLineInput;
use crate::output::CartOperation;
use crate::output::ExpandOperation;
use crate::output::ExpandedItem;
use crate::output::MergeOperation;
use crate::output::PriceAdjustment;
use crate::output::PriceAdjustmentValue;
use crate::output::ImageInput;
use crate::input::InputCartLines;
use serde::{Deserialize, Serialize};
use shopify_function::prelude::*;
use shopify_function::Result;
use std::collections::HashMap;


generate_types!(
    query_path = "./input.graphql",
    schema_path = "./schema.graphql"
);

#[allow(clippy::upper_case_acronyms)]
type URL = String;

#[derive(Clone, Debug, Deserialize)]
struct ZpBundle {
    id: usize,
    rules: Vec<ZpBundleRule>,
}

#[derive(Clone, Debug, Deserialize)]
struct ZpBundleRule {
    productsCount: usize,
    discount: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct ZpBundleItemAttr {
    time: String,
    id: usize,
}

#[derive(Debug, Clone)]
struct SearchResult {
    rule: ZpBundleRule,
    cart_lines: Vec<CartLineInput>,
}

#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let zp_bundles = get_zp_bundles(&input.cart);
    let zp_bundle: ZpBundle = zp_bundles.first().unwrap().clone();
    let bundle_cart_lines: Vec<InputCartLines> = get_bundle_cart_lines(&input.cart, zp_bundle.id);
    let groups = group_items_by_time(bundle_cart_lines.clone());

    let search_results: Vec<SearchResult> = get_search_results(groups, zp_bundle);

    if search_results.is_empty() {
        let no_changes = output::FunctionResult {
            operations: Some(vec![]),
        };

        return Ok(no_changes);
    }

    let merge_operations: Vec<CartOperation> = search_results
        .iter()
        .map(|search_result| {
            let rule: &ZpBundleRule = &search_result.rule;

            // println!("{:?}", &bundle_cart_lines.get(0).unwrap().merchandise.id);

            let parentVariantId: &str = if let ProductVariant(merchandise) = &bundle_cart_lines.get(0).unwrap().merchandise {
                &merchandise.id
            } else {
                ""
            };

            let merge_operation = MergeOperation {
                parent_variant_id: parentVariantId.to_string(),
                cart_lines: search_result.cart_lines.clone(),

                title: None,
                image: None,

                price: Some(PriceAdjustment {
                    percentage_decrease: Some(PriceAdjustmentValue {
                        value: rule.discount.to_string(),
                    }),
                }),
            };

            CartOperation {
                merge: Some(merge_operation),
                expand: None,
            }
        })
        .collect();

    return Ok(output::FunctionResult {
        operations: Some(merge_operations)
    });
}

fn get_zp_bundles(cart: &Cart) -> Vec<ZpBundle> {
    let attribute_value: &Attribute = cart.attribute.as_ref().unwrap();
    let value = attribute_value.value.as_ref().unwrap();
    let zp_bundles: Vec<ZpBundle> = serde_json::from_str(value).unwrap();

    zp_bundles
}

fn get_bundle_cart_lines(cart: &Cart, bundle_id: usize) -> Vec<InputCartLines> {
    return cart.lines
        .clone()
        .into_iter()
        .filter(move |line| {
            if let Some(zp_bundle_attr) = &line.attribute {
                let bundleAttr: ZpBundleItemAttr = serde_json::from_str(zp_bundle_attr.value.as_ref().unwrap()).unwrap();

                // println!("{}, {}", bundleAttr.id, bundle_id);

                return bundleAttr.id == bundle_id;
            }

            return false;
        })
        .collect();
}

fn group_items_by_time(bundle_cart_lines: Vec<InputCartLines>) -> HashMap<String, Vec<InputCartLines>> {
    let mut groups: HashMap<String, Vec<InputCartLines>> = HashMap::new();

    bundle_cart_lines.into_iter().for_each(|cart_line| {
        let line_attr = &cart_line.attribute.clone().unwrap();
        let value = line_attr.value.as_ref().unwrap();
        let bundleAttr: ZpBundleItemAttr = serde_json::from_str(value).unwrap();

        let group = groups.entry(bundleAttr.time).or_insert(vec![]);
        group.push(cart_line);
    });

    // println!("\n\n {:?} \n\n", groups);

    groups
}

fn get_search_results(groups: HashMap<String, Vec<InputCartLines>>, bundle: ZpBundle) -> Vec<SearchResult> {
    let mut results: Vec<SearchResult> = Vec::new();

    groups.iter()
        .for_each(|(_, cartLines)| {
            let rule: std::option::Option<ZpBundleRule> = bundle.rules.get(cartLines.len() - 1).cloned();

            if let Some(r) = rule {
                let lines: Vec<CartLineInput> = cartLines.iter().map(|line| CartLineInput {
                    cart_line_id: line.id.to_string(),
                    quantity: line.quantity,
                }).collect();

                results.push(SearchResult {
                    cart_lines: lines,
                    rule: r
                });
            }
        });

    // println!("\n\n {:?} \n\n", results);

    results
}
