
document.onscroll = () => {
  let e = document.elementsFromPoint(20, 20).find(e => e.className == "account");
  let ghi = document.querySelector("#general-ledger-header .account-info");
  if (ghi) {
    ghi.innerHTML = e ? e.querySelector(".account-info").innerHTML : "";
    ghi.parentElement.style.backgroundColor = e ? getComputedStyle(e.querySelector(".header")).backgroundColor.replace("rgba(0, 0, 0, 0)", "") : "";
  }
}

document.addEventListener("click", (e) => {
  const targetId = location.hash.slice(1);
  if (targetId) {
    const targetEl = document.getElementById(targetId);
    if (targetEl && !targetEl.contains(e.target)) {
      location.hash = "🫶";
    }
  }
  if (e.target.tagName == "H2") {
    e.target.parentElement.classList.toggle("hidden");
  } else if (e.target.classList.contains("name") && e.target.parentElement.parentElement.classList.contains("header")) {
    const parent = e.target.parentElement.parentElement.parentElement;
    if (parent.classList.contains("account") && !parent.classList.contains("leaf")) {
      parent.classList.toggle("collapse");
    }
  }
});


document.addEventListener("input", e => {
  if (e.target.parentElement.classList.contains("budget")) updateBAccount(e.target);
})

window.onload = () => {
  let n = 0;
  document.querySelectorAll(".budget input").forEach(elem => {
    const nn = n;
    elem.id = `input-${nn}`;
    elem.addEventListener("keydown", e => {
      if (e.code == "ArrowDown" || e.code == "KeyJ" || e.code == "Enter" || e.code == "NumpadEnter") {
        e.preventDefault();
        document.getElementById(`input-${nn + 2}`).focus();
      } else if (e.code == "ArrowUp" || e.code == "KeyK") {
        e.preventDefault();
        document.getElementById(`input-${nn - 2}`).focus();
      } else if (e.code == "KeyL" || (elem.value == "" && e.code == "ArrowRight" && nn%2==0)) {
        e.preventDefault();
        document.getElementById(`input-${nn + 1}`).focus();
      } else if (e.code == "KeyH" || (elem.value == "" && e.code == "ArrowLeft" && nn%2 == 1)) {
        e.preventDefault();
        document.getElementById(`input-${nn - 1}`).focus();
      } else if (e.code == "KeyX") {
        e.preventDefault();
        elem.value = "";
      }
    })
    n += 1;
    if (elem.value != "") { updateBAccount(elem) }
  });
}

const updateBAccount = (accountElem) => {
  if (accountElem.value.match(/^\d*[.,]?\d{0,2}$/)) {
    accountElem.classList.remove("bad");
  } else {
    accountElem.classList.add("bad");
  }
  const header = accountElem.parentElement.parentElement;
  const debit = Number(header.querySelector(".debit input").value.replace(',', '.'));
  const credit = Number(header.querySelector(".credit input").value.replace(',', '.'));
  header.querySelector(".budget.sum").innerText = (credit - debit).toFixed(2).replace('.', ',');
  header.parentElement.setAttribute("data-rec-credit", credit);
  header.parentElement.setAttribute("data-rec-debit", debit);

  const parent = header.parentElement//.parentElement;
  if (parent.classList.contains("account")) {
    updateBAccountFooter(parent);
  }
}

const updateBAccountFooter = (accountElem) => {
  let header = accountElem.querySelector("& > .header:has(input)");
  let credit = 0;
  let debit = 0;
  if (header) {
    debit = Number(header.querySelector(".debit input").value.replace(',', '.'));
    credit = Number(header.querySelector(".credit input").value.replace(',', '.'));
  }
  for (child of accountElem.children) {
    credit += Number(child.getAttribute("data-rec-credit"));
    debit += Number(child.getAttribute("data-rec-debit"));
  }
  accountElem.setAttribute("data-rec-credit", credit);
  accountElem.setAttribute("data-rec-debit", debit);

  const footer = accountElem.querySelector("& > .footer");
  // console.log(accountElem, footer);
  if (footer) {
    footer.querySelector(".budget.credit").innerText = credit.toFixed(2).replace('.', ',');
    footer.querySelector(".budget.debit").innerText = debit.toFixed(2).replace('.', ',');
    footer.querySelector(".budget.sum").innerText = (credit - debit).toFixed(2).replace('.', ',');
  }

  const parent = accountElem.parentElement;
  if (parent.classList.contains("account")) {
    updateBAccountFooter(parent);
  }
}

const displayOutput = () => {
  const outputArea = document.getElementById("budget-output");
  outputArea.value = "§ TALOUSARVIO";
  for (let header of document.querySelectorAll(".header[id]")) {
    const accountNumber = header.id.split("-")[1];
    const debit = header.querySelector("& > .budget.debit input").value.trim();
    const credit = header.querySelector("& > .budget.credit input").value.trim();
    let amounts = [];
    if (debit) amounts.push(debit + " DR");
    if (credit) amounts.push(credit + " CR");
    if (amounts.length) {
      outputArea.value += `\n${accountNumber}: ${amounts.join("; ")}`;
    }
  }
  document.getElementById("budget-output-container").classList.toggle("hidden");
}

const hideOutput = () => {
  document.getElementById("budget-output-container").classList.toggle("hidden");
}
