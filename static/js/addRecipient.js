function removeAction(e) {
  targetButton = $(e.target);
  targetButton.prop("disabled", true);
  targetRow = targetButton.closest("tr");
  let cols = targetRow.find("td");

  let data = {
    firstName: cols[0].innerHTML,
    lastName: cols[1].innerHTML,
    address: cols[2].innerHTML,
    giftDate: cols[3].innerHTML,
  };

  axios({
    method: "post",
    url: "/user/delete-recipient",
    data,
    withCredentials: true,
  })
    .then((res) => {
      if (res && res.status === 200) {
        targetRow.remove();
      } else {
        targetButton.prop("disabled", false);

        if (res && res.data && res.data.message) {
          console.error(res.data.message);
        }
      }
    })
    .catch((e) => {
      targetButton.prop("disabled", false);
      console.error(e);
    });
}

$(document).ready(
  $("body").on("click", "button.deleteRecipientButton", removeAction)
);

//   $("button.deleteRecipientButton").click(function () {
//     let row = $(this).closest("tr");
//     let cols = row.find("td");

//     let data = {
//       firstName: cols[0].innerHTML,
//       lastName: cols[1].innerHTML,
//       address: cols[2].innerHTML,
//       giftDate: cols[3].innerHTML,
//     };
//     console.log(data);
//   });

// wait 5 seconds before allowing another click

const submit = document.getElementById("submitRecipient");
// streams
const submit$ = rxjs.fromEvent(submit, "click");

submit$
  .pipe(
    rxjs.operators.map((i) => i.currentTarget.value),
    rxjs.operators.throttleTime(5000)
  )
  .subscribe(addRecipient);

function addRecipient() {
  let data = {
    firstName: escape($("#firstName").val()),
    lastName: escape($("#lastName").val()),
    address: escape($("#address").val()),
    giftDate: escape($("#giftDate").val()),
  };

  let formError = false;

  if (data.firstName === "") {
    formError = true;
    $("#firstName").addClass("is-invalid");
  } else {
    $("#firstName").removeClass("is-invalid");
  }
  if (data.lastName === "") {
    formError = true;
    $("#lastName").addClass("is-invalid");
  } else {
    $("#lastName").removeClass("is-invalid");
  }
  if (data.address === "") {
    formError = true;
    $("#address").addClass("is-invalid");
  } else {
    $("#address").removeClass("is-invalid");
  }

  if (data.giftDate === "") {
    formError = true;
    $("#giftDate").addClass("is-invalid");
  } else {
    $("#giftDate").removeClass("is-invalid");
  }

  if (formError) return;
  axios({
    method: "post",
    url: "/user/add-recipient",
    data,
  })
    .then((res) => {
      if (res && res.status === 200) {
        $("#recipientList > tbody:last-child").append(`<tr>
      <td>${data.firstName}</td>
      <td>${data.lastName}</td>
      <td>${data.address}</td>
      <td>${data.giftDate}</td>
      <td><button class="deleteRecipientButton btn btn-danger">Delete</button></td>
    </tr>`);
      }

      $("#giftDate").val("");
      $("#address").val("");
      $("#lastName").val("");
      $("#firstName").val("");
    })
    .catch((e) => console.error(e));
}
