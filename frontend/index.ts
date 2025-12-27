const isLoggedIn = document.cookie.split(';').some(c => c.trim().startsWith("coco_token"))

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
        Object.entries(electionsById).sort().map(([_, value]) => value).forEach(createElectionForm)
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
    var errorMessageElement = document.getElementById("elections-error-message")
    if (errorMessageElement instanceof HTMLParagraphElement) {
        errorMessageElement.innerHTML = "Error: could not load election data."
    }
}

function createElectionForm(election: Election) {
    var electionsDiv = document.getElementById("elections")
    
    const electionFormId = `election-${election.id}`
    
    var electionLabel = document.createElement("label")
    electionLabel.htmlFor = electionFormId
    electionLabel.textContent = election.name
    electionsDiv?.appendChild(electionLabel)
    
    var electionForm = document.createElement("form")
    electionForm.id = electionFormId
    electionsDiv?.appendChild(electionForm)

    const submitButtonId = `election-${election.id}-submit`
    const radioButtonGroupName = `election-${election.id}-selected-ballot-item-id`

    Object.entries(election.ballotItemsById).sort().map(([_, value]) => value).forEach((ballotItem) => {
        const ballotItemInputId = `election-${election.id}-ballot-item-${ballotItem.id}`

        var ballotItemInput = document.createElement("input")
        ballotItemInput.type = "radio"
        ballotItemInput.id = ballotItemInputId
        ballotItemInput.value = ballotItem.id.toString()
        ballotItemInput.name = radioButtonGroupName
        ballotItemInput.addEventListener("change", () => {
            var submitButton = document.getElementById(submitButtonId)

            if (submitButton instanceof HTMLInputElement) {
                submitButton.disabled = false
            }
        })

        var label = document.createElement("label")
        label.textContent = ballotItem.name
        label.htmlFor = ballotItemInputId

        electionForm.appendChild(ballotItemInput)
        electionForm.appendChild(label)
        electionForm.appendChild(document.createElement("br"))
    })

    var electionIdHiddenInput = document.createElement("input")
    electionIdHiddenInput.type = "hidden"
    electionIdHiddenInput.name = "election-id"
    electionIdHiddenInput.value = election.id.toString()
    electionForm.appendChild(electionIdHiddenInput)

    var submitButton = document.createElement("input")
    submitButton.id = submitButtonId
    submitButton.type = "submit"
    submitButton.value = "Submit"
    submitButton.disabled = true
    electionForm.appendChild(submitButton)

    var electionMessageElementId = `election-${election.id}-message`
    var electionMessageElement = document.createElement("p")
    electionMessageElement.id = electionMessageElementId
    electionForm.appendChild(electionMessageElement)

    electionForm.addEventListener("submit", async (event) => {
        event.preventDefault()

        const data = new FormData(electionForm)
        const dataObject = Object.fromEntries(data)
        const dataParsed = {
            electionId: Number(dataObject["election-id"]),
            selectedBallotItemId: Number(dataObject[radioButtonGroupName]),
        }

        const response = await fetch("/elections/vote", {
            method: "POST",
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(dataParsed)
        })

        if (response.ok || response.status === 403) {
            const inputElements = electionForm.querySelectorAll("input") as NodeListOf<HTMLInputElement>
            for (var inputElement of inputElements.values()) {
                inputElement.disabled = true
            }
        }
        
        if (response.ok) {
            var errorMessageElement = document.getElementById(electionMessageElementId)
            if (errorMessageElement instanceof HTMLParagraphElement) {
                errorMessageElement.innerHTML = "Voting successful." 
            }
        } else if (response.status === 401) {
            window.location.href = "/login"
        } else if ([403, 404, 500].includes(response.status)) {
            var errorMessageElement = document.getElementById(electionMessageElementId)
            if (errorMessageElement instanceof HTMLParagraphElement) {
                const responseText = await response.text()
                errorMessageElement.innerHTML = "Error: " + responseText
            }
        } else if (!response.ok) {
            var errorMessageElement = document.getElementById(electionMessageElementId)
            if (errorMessageElement instanceof HTMLParagraphElement) {
                errorMessageElement.innerHTML = "Unexpected error: " + response.status   
            }
        }
    })
}

// function getCookie(key: string): string | undefined {
//     return document.cookie.split(';').find(c => c.trim().startsWith(key))?.trim()
// }
