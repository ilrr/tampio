
document.onscroll = () => {
  let e = document.elementsFromPoint(20, 20).find(e => e.className == "account");
  let ghi = document.querySelector("#general-ledger-header .account-info");
  ghi.innerHTML = e ? e.querySelector(".account-info").innerHTML : "";
  ghi.parentElement.style.backgroundColor = e ? getComputedStyle(e.querySelector(".header")).backgroundColor.replace("rgba(0, 0, 0, 0)", "") : "";
}

document.addEventListener("click", (e) => {
  const targetId = location.hash.slice(1);
  const targetEl = document.getElementById(targetId);

  if (targetEl && !targetEl.contains(e.target)) {
    location.hash = "🫶";
  }
  if (e.target.tagName == "H2") {
    e.target.parentElement.classList.toggle("hidden");
  }
});
