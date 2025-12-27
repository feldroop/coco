const isLoggedIn = document.cookie.split(';').some(c => c.trim().startsWith("coco_admin_token"))

if (!isLoggedIn) {
    window.location.href = "/admin/login"
}
