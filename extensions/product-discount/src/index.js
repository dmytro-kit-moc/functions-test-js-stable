// @ts-check
import {DiscountApplicationStrategy} from "../generated/api";

/**
 * @typedef {import("../generated/api").InputQuery} InputQuery
 * @typedef {import("../generated/api").FunctionResult} FunctionResult
 */

/**
 * @type {FunctionResult}
 */
const EMPTY_DISCOUNT = {
    discountApplicationStrategy: DiscountApplicationStrategy.First,
    discounts: [],
};

export default (input) => {
    return EMPTY_DISCOUNT;

    const config = JSON.parse(input?.discountNode?.metafield?.value ?? '{}');
    const { quantity, percentage } = config;

    if (!(quantity && percentage)) {
        return EMPTY_DISCOUNT;
    }

    const targets = input.cart.lines
        .filter((line) => {
            const isProduct = line.merchandise.__typename == 'ProductVariant'
            return isProduct && line.quantity >= quantity;
        })
        .map((line) => {
            return ({
                productVariant: {
                    id: line.merchandise.id
                }
            });
        });

    if (!targets.length) {
        console.error("No cart lines qualify for volume discount.");
        return EMPTY_DISCOUNT;
    }

    const discounts = [
        createPercentageDiscount(percentage, targets),
    ];

    // Works only first/max discount!
    // https://community.shopify.com/c/shopify-functions/shopify-functions-product-discount-api-multiple-discounts/td-p/1693369
    // const discounts = [
    //     createFixedDiscount(1.99, targets),
    //     createPercentageDiscount(percentage, targets)
    // ];

    // Does not work!
    // const discounts = [
    //     createFixedDiscount(1.99, [targets[0]]),
    //     createPercentageDiscount(percentage, [targets[1]])
    // ];

    return {
        discountApplicationStrategy: DiscountApplicationStrategy.First,
        discounts
    };
};

function createPercentageDiscount(value, targets) {
    return {
        targets,
        message: `-${value}% off`, // or discount name will be displayed
        value: {
            percentage: {
                value
            }
        }
    };
}

function createFixedDiscount(value, targets) {
    return {
        targets,
        message: `-$${value} off`, // or discount name will be displayed
        value: {
            fixedAmount: {
                amount: value,
                appliesToEachItem: false
            }
        }
    };
}
