const isLoggedIn = document.cookie
  .split(";")
  .some((c) => c.trim().startsWith("coco_admin_token"));

if (!isLoggedIn) {
  window.location.href = "/admin/login";
}

var addBallotItemButton = document.getElementById("add-ballot-item");
var ballotItemsListElement = document.getElementById("ballot-items");
var ballotItemId = 0;

addBallotItemButton?.addEventListener("click", (event) => {
  event.preventDefault();

  var listItem = document.createElement("li");
  ballotItemsListElement?.appendChild(listItem);

  var ballotItemInput = document.createElement("input");
  ballotItemInput.type = "text";
  ballotItemInput.name = `ballot-item-${ballotItemId.toString()}`;

  var deleteBallotItemButton = document.createElement("button");
  deleteBallotItemButton.innerText = "Delete";
  deleteBallotItemButton.addEventListener("click", (event) => {
    event.preventDefault();
    listItem.remove();
  });

  listItem.appendChild(ballotItemInput);
  listItem.appendChild(deleteBallotItemButton);

  ballotItemInput.select();

  ballotItemId += 1;
});

var addElectionForm = document.getElementById("create-election-form");

addElectionForm?.addEventListener("submit", async (event) => {
  event.preventDefault();

  if (addElectionForm instanceof HTMLFormElement) {
    const createElectionData = new FormData(addElectionForm);

    var createElectionDataObject = {
      name: createElectionData.get("name")?.toString(),
      ballotItems: [] as string[],
    };

    if (createElectionDataObject.name?.length === 0) {
      var message = document.getElementById("create-election-form-message");
      if (message instanceof HTMLParagraphElement) {
        message.textContent = "Error: Election title cannot be empty.";
        return;
      }
    }

    for (const [key, value] of createElectionData) {
      if (key.startsWith("ballot-item-")) {
        const valueString = value.toString();

        if (valueString.length === 0) {
          var message = document.getElementById("create-election-form-message");
          if (message instanceof HTMLParagraphElement) {
            message.textContent = "Error: Ballot item name cannot be empty.";
            return;
          }
        }

        createElectionDataObject.ballotItems.push(valueString);
      }
    }

    const response = await fetch("/admin/create-election", {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(createElectionDataObject),
    });

    if (response.ok) {
      addElectionForm.reset();
      var message = document.getElementById("create-election-form-message");
      if (message instanceof HTMLParagraphElement) {
        message.textContent = "Created election successfully.";
      }
    } else if (response.status === 401) {
      window.location.href = "/admin/login";
    } else {
      var message = document.getElementById("create-election-form-message");
      if (message instanceof HTMLParagraphElement) {
        message.textContent = "Unexpected error: " + response.status;
      }
    }
  }
});
