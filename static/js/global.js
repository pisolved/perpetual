function sanitize(string) {
  const map = {
    "<": "&lt;",
    ">": "&gt;",
  };
  const reg = /[<>]/gi;
  return string.replace(reg, (match) => map[match]);
}
