#![allow(dead_code, unused_variables, unused_imports, unreachable_code)]

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
use sha2::{Digest, Sha256};


generate_types!(
    query_path = "./input.graphql",
    schema_path = "./schema.graphql"
);

#[allow(clippy::upper_case_acronyms)]
type URL = String;

#[derive(Clone, Debug, Deserialize)]
struct ZpBundle {
    pub id: usize,
    pub rules: Vec<ZpBundleRule>,
}

#[derive(Clone, Debug, Deserialize)]
struct ZpBundleRule {
    pub parent_product_id: String,
    pub title: Option<String>,
    pub items: Vec<ZpBundleRuleItem>,
    pub discount: ZpBundleDiscount
}

#[derive(Clone, Debug, Deserialize)]
struct ZpBundleRuleItem {
    pub id: String,
    pub quantity: usize
}

#[derive(Clone, Debug, Deserialize)]
struct ZpBundleDiscount {
    pub value: usize,
    pub discount_type: String,
}

#[derive(Debug, Clone)]
struct SearchResult {
    rule: ZpBundleRule,
    cart_lines: Vec<CartLineInput>,
}

#[shopify_function]
fn function(input: input::ResponseData) -> Result<output::FunctionResult> {
    let zp_bundles = get_zp_bundles(&input.cart);
    let mut zp_bundle: ZpBundle = zp_bundles.first().unwrap().clone();
    let mut bundle_cart_lines: Vec<InputCartLines> = get_bundle_cart_lines(&input.cart, zp_bundle.id);

    let search_results: Vec<SearchResult> = check_rules(&mut bundle_cart_lines, &mut zp_bundle.rules);

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

            let bundle_title: String = if rule.title.is_some() {
                rule.title.as_ref().unwrap().to_string()
            } else {
                let items_count_str = if rule.items.len() == 1 {
                    String::from("1 item")
                } else {
                    format!("{} items", rule.items.len())
                };

                format!(
                    "My custom bundle: {} (-{}% off)",
                    items_count_str,
                    rule.discount.value
                )
            };

            let merge_operation = MergeOperation {
                parent_variant_id: String::from(rule.parent_product_id.to_string()),
                title: Some(bundle_title),
                cart_lines: search_result.cart_lines.clone(),

                image: None,
                // Some(ImageInput {
                //     url: "https://cdn.shopify.com/s/files/1/0458/3856/5534/products/LEZYNE_PATCH-TOOL_GOLD_WEB_05486981-3d2c-41ff-b73a-18c6fc85d02f.jpg?v=1651743108".to_string()
                // }),

                price: Some(PriceAdjustment {
                    percentage_decrease: Some(PriceAdjustmentValue {
                        value: rule.discount.value.to_string(),
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
            if let Some(zp_bundle_id_attr) = &line.attribute {
                let value = zp_bundle_id_attr.value.as_ref().unwrap();
                return value == &bundle_id.to_string();
            }

            return false;
        })
        .collect();
}

fn check_rules(bundle_cart_lines: &mut Vec<InputCartLines>, rules: &mut Vec<ZpBundleRule>) -> Vec<SearchResult> {
    let mut search_results: Vec<SearchResult> = Vec::new();

    rules.sort_by(|a, b| b.items.len().cmp(&a.items.len()));

    for rule in rules.iter() {
        let mut c_lines: Vec<CartLineInput> = Vec::new();

        rule.items.iter().for_each(|item| {
            let cart_line = bundle_cart_lines
                .iter()
                .find(|&line| {
                    if let ProductVariant(merchandise) = &line.merchandise {
                        return merchandise.id == item.id && line.quantity >= item.quantity.try_into().unwrap();
                    }

                    return false
                });

            // println!("\n({:?}, \n\n\n {:?} \n\n {:?} ===", item, bundle_cart_lines, cart_line);

            if cart_line.is_some() {
                c_lines.push(CartLineInput {
                    cart_line_id: cart_line.unwrap().id.to_string(),
                    quantity: item.quantity as i64,
                });
            }
        });

        let is_rule_matched = c_lines.len() == rule.items.len();

        if is_rule_matched {
            search_results.push(SearchResult {
                rule: rule.clone(),
                cart_lines: c_lines.clone()
            });

            c_lines.iter().for_each(|line| {
                let index = bundle_cart_lines.iter().position(|x| *x.id == line.cart_line_id).unwrap();
                bundle_cart_lines.remove(index);
            });
        }

        if bundle_cart_lines.is_empty() {
            break;
        }
    }

    return search_results;
}

/*
    {
        id: 1,
        rules: [
            {
                parent_product_id: 'gid://shopify/ProductVariant/42539430871198',
                title: 'All products: -50%',

                items: [
                    { id: 'gid://shopify/ProductVariant/40799008719006', quantity: 1 },
                    { id: 'gid://shopify/ProductVariant/41707097620638', quantity: 1 },
                    { id: 'gid://shopify/ProductVariant/40799008227486', quantity: 1 },
                    { id: 'gid://shopify/ProductVariant/41707097227422', quantity: 1 },
                ],

                discount: {
                    value: 50,
                    discount_type: 'percentage'
                }
            },

            {
                parent_product_id: 'gid://shopify/ProductVariant/42539430871198',

                items: [
                    { id: 'gid://shopify/ProductVariant/40799008719006', quantity: 1 },
                    { id: 'gid://shopify/ProductVariant/40799008227486', quantity: 1 },
                ],

                discount: {
                    value: 20,
                    discount_type: 'percentage'
                }
            },

            {
                parent_product_id: 'gid://shopify/ProductVariant/42539430871198',

                items: [
                    { id: 'gid://shopify/ProductVariant/41707097620638', quantity: 2 },
                ],

                discount: {
                    value: 5,
                    discount_type: 'percentage'
                }
            },

            {
                parent_product_id: 'gid://shopify/ProductVariant/42539430871198',

                items: [
                    { id: 'gid://shopify/ProductVariant/41707097620638', quantity: 2 },
                    { id: 'gid://shopify/ProductVariant/40799008227486', quantity: 1 },
                ],

                discount: {
                    value: 15,
                    discount_type: 'percentage'
                }
            },
        ]
    }
*/
