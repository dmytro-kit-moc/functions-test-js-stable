export class BlockView {
    _bundleBlockEl;
    _cartService;

    constructor({ bundleBlockEl, cartService }) {
        this._bundleBlockEl = bundleBlockEl;
        this._cartService = cartService;

        this.updateAttributes();

        document.querySelectorAll('button[data-productid]').forEach((buttonEl) => {
            buttonEl.addEventListener('click', () => {
                const productId = buttonEl.dataset.productid
                console.error(productId);
                this._cartService.addProduct(productId, 1)
            })
        })
    }

    updateAttributes() {
        this._cartService.updateAttributes({
            zpBundles: JSON.stringify([getBundleConfig()])
        })
    }
}

function getBundleConfig() {
    return {
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
}
