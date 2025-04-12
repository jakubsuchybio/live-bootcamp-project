const loginLink = document.getElementById("login-link");
const logoutLink = document.getElementById("logout-link");
const protectImg = document.getElementById("protected-img");
// Get the current prefix from the path or use an empty string
const currentPrefix = window.location.pathname.includes('/app') ? '/app' : '';

// Store the original image src to use as fallback
const originalImgSrc = protectImg.getAttribute('src');

logoutLink.addEventListener("click", (e) => {
    e.preventDefault();

    let url = logoutLink.href;

    fetch(url, {
        method: 'POST',
        credentials: 'include', // This will include cookies in the request
    }).then(response => {
        if (response.ok) {
            loginLink.style.display = "block";
            logoutLink.style.display = "none";
            protectImg.src = currentPrefix + "/assets/default.jpg";
        } else {
            alert("Failed to logout");
        }
    });
    });
    
    (() => {
    fetch(currentPrefix + '/protected').then(response => {
        if (response.ok) {
            loginLink.style.display = "none";
            logoutLink.style.display = "block";

            response.json().then(data => {
                let img_url = data.img_url;
                if (img_url !== undefined && img_url !== null && img_url !== "") {
                    protectImg.src = img_url;
                } else {
                    protectImg.src = currentPrefix + "/assets/default.jpg";
                }
                            });
                        } else {
                            loginLink.style.display = "block";
                            logoutLink.style.display = "none";
                            protectImg.src = currentPrefix + "/assets/default.jpg";
        }
    });
})();