// elem ref
const username = document.getElementById("username");

// streams
const username$ = rxjs.fromEvent(username, "keyup");

// wait .5s between keyups to emit current value
username$
  .pipe(
    rxjs.operators.map((i) => i.currentTarget.value),
    rxjs.operators.debounceTime(500)
  )
  .subscribe(checkUnique);
// elem ref
const email = document.getElementById("email");

// streams
const email$ = rxjs.fromEvent(email, "keyup");

// wait .5s between keyups to emit current value
email$
  .pipe(
    rxjs.operators.map((i) => i.currentTarget.value),
    rxjs.operators.debounceTime(500)
  )
  .subscribe(checkUnique);

function checkUnique() {
  axios({
    method: "post",
    url: "/check-unique",
    data: {
      username: $("#username").val(),
      email: $("#email").val(),
    },
  })
    .then((res) => console.log(res))
    .catch((e) => console.error(e));
}
