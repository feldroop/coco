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
    }
} catch (error) {
    console.error("Error fetching or parsing for elections:", error)
}

function createElectionForm(election: Election) {
    var electionLabel = document.getElementById("election-label")
    if (electionLabel instanceof HTMLLabelElement) {
        electionLabel.textContent = election.name
    }

    const form = document.forms.namedItem("election")
    const ballotItemFieldset = document.getElementById("ballot-items")

    if (form && ballotItemFieldset) {
        // const a = election.ballotItemsById;

        for (const ballotItemId in election.ballotItemsById) {
            const ballotItem = election.ballotItemsById[ballotItemId]!
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
        }

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

            console.log(response)
        })
    }
}

// function getCookie(key: string): string | undefined {
//     return document.cookie.split(';').find(c => c.trim().startsWith(key))?.trim()
// }
