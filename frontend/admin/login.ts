const form = document.forms.namedItem('admin-login-form')

form?.addEventListener('submit', async (event) => {
    event.preventDefault()
    const data = new FormData(form)

    const response = await fetch('/admin/start-session', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(Object.fromEntries(data)),
    })

    if (response.status === 401) {
        var errorMessageElement = document.getElementById(
            'admin-login-form-error-message'
        )
        if (errorMessageElement instanceof HTMLParagraphElement) {
            errorMessageElement.innerHTML = 'Wrong password.'
        }
    } else if (!response.ok) {
        var errorMessageElement = document.getElementById(
            'admin-login-form-error-message'
        )
        if (errorMessageElement instanceof HTMLParagraphElement) {
            errorMessageElement.innerHTML =
                'Unexpected error: ' + response.status
        }
    } else {
        window.location.href = '/admin'
    }
})
