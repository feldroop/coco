const form = document.forms.namedItem('login-form')

form?.addEventListener('submit', async (event) => {
    event.preventDefault()
    const data = new FormData(form)

    const response = await fetch('/participants/add', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(Object.fromEntries(data)),
    })

    if (response.status === 401) {
        let errorMessageElement = document.getElementById(
            'login-form-error-message'
        )
        if (errorMessageElement instanceof HTMLParagraphElement) {
            errorMessageElement.innerHTML = 'Wrong password.'
        }
    } else if (!response.ok) {
        let errorMessageElement = document.getElementById(
            'login-form-error-message'
        )
        if (errorMessageElement instanceof HTMLParagraphElement) {
            errorMessageElement.innerHTML =
                'Unexpected error: ' + response.status
        }
    } else {
        window.location.href = '/'
    }
})
