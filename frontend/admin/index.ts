const isLoggedIn = document.cookie
    .split(';')
    .some((c) => c.trim().startsWith('coco_admin_token'))

if (!isLoggedIn) {
    window.location.href = '/admin/login'
}

var addBallotItemButton = document.getElementById('add-ballot-item')
var ballotItemsListElement = document.getElementById('ballot-items')
var ballotItemId = 0

addBallotItemButton?.addEventListener('click', (event) => {
    event.preventDefault()

    var listItem = document.createElement('li')
    ballotItemsListElement?.appendChild(listItem)

    var ballotItemInput = document.createElement('input')
    ballotItemInput.type = 'text'
    ballotItemInput.name = `ballot-item-${ballotItemId.toString()}`

    var deleteBallotItemButton = document.createElement('button')
    deleteBallotItemButton.innerText = 'Delete'
    deleteBallotItemButton.addEventListener('click', (event) => {
        event.preventDefault()
        listItem.remove()
    })

    listItem.appendChild(ballotItemInput)
    listItem.appendChild(deleteBallotItemButton)

    ballotItemInput.select()

    ballotItemId += 1
})

var addElectionForm = document.getElementById('create-election-form')

addElectionForm?.addEventListener('submit', async (event) => {
    event.preventDefault()

    if (addElectionForm instanceof HTMLFormElement) {
        const createElectionData = new FormData(addElectionForm)

        var createElectionDataObject = {
            name: createElectionData.get('name')?.toString(),
            ballotItems: [] as string[],
        }

        if (createElectionDataObject.name?.length === 0) {
            var message = document.getElementById(
                'create-election-form-message'
            )
            if (message instanceof HTMLParagraphElement) {
                message.textContent = 'Error: Election title cannot be empty.'
                return
            }
        }

        for (const [key, value] of createElectionData) {
            if (key.startsWith('ballot-item-')) {
                const valueString = value.toString()

                if (valueString.length === 0) {
                    var message = document.getElementById(
                        'create-election-form-message'
                    )
                    if (message instanceof HTMLParagraphElement) {
                        message.textContent =
                            'Error: Ballot item name cannot be empty.'
                        return
                    }
                }

                createElectionDataObject.ballotItems.push(valueString)
            }
        }

        const response = await fetch('/admin/create-election', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify(createElectionDataObject),
        })

        if (response.ok) {
            addElectionForm.reset()
            var message = document.getElementById(
                'create-election-form-message'
            )
            if (message instanceof HTMLParagraphElement) {
                message.textContent = 'Created election successfully.'
            }
        } else if (response.status === 401) {
            window.location.href = '/admin/login'
        } else {
            var message = document.getElementById(
                'create-election-form-message'
            )
            if (message instanceof HTMLParagraphElement) {
                message.textContent = 'Unexpected error: ' + response.status
            }
        }
    }
})

interface BallotItem {
    id: number
    name: string
}

interface Election {
    id: number
    name: string
    ballotItemsById: Record<number, BallotItem>
}

const intervalId = setInterval(updateAndRenderElections, 5000)
// AI told me to add this clean-up
window.addEventListener('beforeunload', () => {
    clearInterval(intervalId)
})

await updateAndRenderElections()

async function updateAndRenderElections() {
    try {
        const electionsResponse = await fetch('/elections')

        if (electionsResponse.ok) {
            const electionsById: Record<number, Election> =
                await electionsResponse.json()

            var electionsDiv = document.getElementById('elections')
            electionsDiv?.replaceChildren()

            Object.entries(electionsById)
                .sort()
                .map(([_, value]) => value)
                .forEach(createElectionDisplay)
        } else if (electionsResponse.status === 401) {
            window.location.href = '/admin/login'
        } else {
            var errorMessageElement = document.getElementById(
                'elections-error-message'
            )
            if (errorMessageElement instanceof HTMLParagraphElement) {
                errorMessageElement.innerHTML =
                    'Unexpected error: ' + electionsResponse.status
            }
        }
    } catch (error) {
        var errorMessageElement = document.getElementById(
            'elections-error-message'
        )
        if (errorMessageElement instanceof HTMLParagraphElement) {
            errorMessageElement.innerHTML =
                'Error: could not load election data.'
        }
    }
}

function createElectionDisplay(election: Election) {
    var electionsDiv = document.getElementById('elections')

    const electionFormId = `election-${election.id}`

    var electionLabel = document.createElement('label')
    electionLabel.htmlFor = electionFormId
    electionLabel.textContent = election.name
    electionsDiv?.appendChild(electionLabel)

    var electionForm = document.createElement('form')
    electionForm.id = electionFormId
    electionsDiv?.appendChild(electionForm)

    const submitButtonId = `election-${election.id}-submit`
    const radioButtonGroupName = `election-${election.id}-selected-ballot-item-id`

    Object.entries(election.ballotItemsById)
        .sort()
        .map(([_, value]) => value)
        .forEach((ballotItem) => {
            const ballotItemInputId = `election-${election.id}-ballot-item-${ballotItem.id}`

            var ballotItemInput = document.createElement('input')
            ballotItemInput.type = 'radio'
            ballotItemInput.id = ballotItemInputId
            ballotItemInput.value = ballotItem.id.toString()
            ballotItemInput.name = radioButtonGroupName
            ballotItemInput.addEventListener('change', () => {
                var submitButton = document.getElementById(submitButtonId)

                if (submitButton instanceof HTMLInputElement) {
                    submitButton.disabled = false
                }
            })

            var label = document.createElement('label')
            label.textContent = ballotItem.name
            label.htmlFor = ballotItemInputId

            electionForm.appendChild(ballotItemInput)
            electionForm.appendChild(label)
            electionForm.appendChild(document.createElement('br'))
        })

    var electionIdHiddenInput = document.createElement('input')
    electionIdHiddenInput.type = 'hidden'
    electionIdHiddenInput.name = 'election-id'
    electionIdHiddenInput.value = election.id.toString()
    electionForm.appendChild(electionIdHiddenInput)
}
