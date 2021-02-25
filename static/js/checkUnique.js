// elem ref
const username = document.getElementById("username");
const email = document.getElementById("email");

// streams
const username$ = rxjs.fromEvent(username, "keyup");
const email$ = rxjs.fromEvent(email, "keyup");

// wait .5s between keyups to emit current value
username$
  .pipe(
    rxjs.operators.map((i) => i.currentTarget.value),
    rxjs.operators.debounceTime(500)
  )
  .subscribe(checkUnique);

email$
  .pipe(
    rxjs.operators.map((i) => i.currentTarget.value),
    rxjs.operators.debounceTime(500)
  )
  .subscribe(checkUnique);

const constraints = {
  username: {
    presence: true,
    length: {
      minimum: 3,
      message: "must be at least 3 characters",
    },
  },
  email: {
    presence: true,
    from: {
      email: true,
    },
  },
  password: {
    presence: true,
  },
  firstName: {
    presence: true,
  },
  lastName: {
    presence: true,
  },
  phone: {
    presence: true,
  },
};

function checkUnique() {
  axios({
    method: "post",
    url: "/check-unique",
    data: {
      username: $("#username").val(),
      email: $("#email").val(),
    },
  })
    .then((res) => {
      console.log(res.data);
      if (res && res.data) {
        if (!res.data.username) {
          $("#username").addClass("is-invalid");
          $("#submitButton").prop("disabled", true);
        } else {
          $("#username").removeClass("is-invalid");
          $("#submitButton").prop("disabled", false);
        }
        if (!res.data.email) {
          $("#email").addClass("is-invalid");
          $("#submitButton").prop("disabled", true);
        } else {
          $("#email").removeClass("is-invalid");
          $("#submitButton").prop("disabled", false);
        }
      }
    })
    .catch((e) => console.error(e));
}
