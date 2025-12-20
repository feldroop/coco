const form = document.forms.namedItem("login-form")!;

form.addEventListener("submit", async (event) => {
    event.preventDefault();

    const password = form.elements.namedItem("password") as HTMLInputElement;

    const url = window.location.origin + "/participants/add";
    const requestBody = { "password": password.value };

    const response = await fetch(url, {
        method: "POST",
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(requestBody)
    });

    if (response.status === 401) {
        document.getElementById("errorMessage")!.innerHTML = "Wrong password.";
    } else if (!response.ok) {
        document.getElementById("errorMessage")!.innerHTML = "Unexpected error: " + response.status;
    } else {
        window.location.href = "/";
    }
});
