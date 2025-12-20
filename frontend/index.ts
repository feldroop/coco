document.addEventListener('DOMContentLoaded', () => {
    const isLoggedIn = document.cookie.split(';').some(c => c.trim() === 'logged_in=true');

    if (!isLoggedIn) {
        window.location.href = '/login';
    }
});
