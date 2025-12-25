const isLoggedIn = document.cookie.split(';').some(c => c.trim().startsWith("coco_access_token"))

if (!isLoggedIn) {
    window.location.href = "/login"
}

interface BallotItem {
    id: number
    name: string
}

interface Election {
    id: number
    name: string
    ballotItemsById: Record<number, BallotItem>
}

try {  
    const electionsResponse = await fetch("/elections")

    if (electionsResponse.ok) {
        const electionsById: Record<number, Election> = await electionsResponse.json()
        const electionId = 0;
        const election = electionsById[electionId]!
        createElectionForm(election)
    } else if (electionsResponse.status === 401) {
        window.location.href = "/login"
    } else {
        var errorMessageElement = document.getElementById("election-form-error-message")
        if (errorMessageElement instanceof HTMLParagraphElement) {
            errorMessageElement.innerHTML = "Unexpected error: " + electionsResponse.status
        }
    }
} catch (error) {
    console.error("Error fetching or parsing for elections:", error)
    var errorMessageElement = document.getElementById("election-form-error-message")
    if (errorMessageElement instanceof HTMLParagraphElement) {
        errorMessageElement.innerHTML = "Error: could not load election data."
    }
}

function createElectionForm(election: Election) {
    var electionLabel = document.getElementById("election-label")
    if (electionLabel instanceof HTMLLabelElement) {
        electionLabel.textContent = election.name
    }

    const form = document.forms.namedItem("election")
    const ballotItemFieldset = document.getElementById("ballot-items")

    if (form && ballotItemFieldset) {
        Object.values(election.ballotItemsById).forEach((ballotItem) => {
            const ballotItemInputId = `ballot-item-${ballotItem.id}`
    
            var ballotItemInput = document.createElement("input")
            ballotItemInput.type = "radio"
            ballotItemInput.id = ballotItemInputId
            ballotItemInput.value = ballotItem.id.toString()
            ballotItemInput.name = "selectedBallotItemId"
            ballotItemInput.addEventListener("change", () => {
                var submitButton = document.getElementById("submit-election-vote")
    
                if (submitButton instanceof HTMLInputElement) {
                    submitButton.disabled = false
                }
            })
    
            var label = document.createElement("label")
            label.textContent = ballotItem.name
            label.htmlFor = ballotItemInputId
    
            ballotItemFieldset.appendChild(ballotItemInput)
            ballotItemFieldset.appendChild(label)
            ballotItemFieldset.appendChild(document.createElement("br"))
        })

        var electionIdElement = document.createElement("input")
        electionIdElement.type = "hidden"
        electionIdElement.name = "electionId"
        electionIdElement.value = election.id.toString()
        form.appendChild(electionIdElement)

        form.addEventListener("submit", async (event) => {
            event.preventDefault()

            const data = new FormData(form)
            const dataObject = Object.fromEntries(data)
            const dataParsed = {
                electionId: Number(dataObject["electionId"]),
                selectedBallotItemId: Number(dataObject["selectedBallotItemId"]),
            }

            const response = await fetch("/elections/vote", {
                method: "POST",
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify(dataParsed)
            })

            if (response.ok || response.status === 403) {
                const radioButtons = form.querySelectorAll("input") as NodeListOf<HTMLInputElement>
                for (var radioButton of radioButtons.values()) {
                    radioButton.disabled = true
                }
            }
            
            if (response.status === 401) {
                window.location.href = "/login"
            } else if ([403, 404, 500].includes(response.status)) {
                var errorMessageElement = document.getElementById("election-form-error-message")
                if (errorMessageElement instanceof HTMLParagraphElement) {
                    const responseText = await response.text()
                    errorMessageElement.innerHTML = "Error: " + responseText
                }
            } else if (!response.ok) {
                var errorMessageElement = document.getElementById("election-form-error-message")
                if (errorMessageElement instanceof HTMLParagraphElement) {
                    errorMessageElement.innerHTML = "Unexpected error: " + response.status   
                }
            }
        })
    }
}

// function getCookie(key: string): string | undefined {
//     return document.cookie.split(';').find(c => c.trim().startsWith(key))?.trim()
// }
