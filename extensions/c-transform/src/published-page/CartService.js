export class CartService {
    updateAttributes(cartAttributes) {
        fetch('/cart/update.js', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ 'attributes': cartAttributes })
        }).then(async () => {
            const cart = await this.fetchCart()
            console.error(cart.attributes);
        }).catch(error => {
            console.error('Error:', error);
        });
    }

    fetchCart() {
        return fetch('/cart.js').then(response => response.json())
    }

    addProduct(productId, bundleId) {
        return fetch(`/cart/add.js`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                id: productId,
                form_type: 'product',
                properties: {
                    zpBundleId: bundleId,
                }
            })
        })
            .then(response => response.json())
            .then(data => {
                console.log(data);
            })
            .catch(error => {
                console.error(error);
            });
    }
}
