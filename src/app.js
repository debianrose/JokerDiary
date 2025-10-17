const { invoke } = window.__TAURI__.tauri;

document.getElementById('greet-btn').addEventListener('click', async () => {
    try {
        const result = await invoke('greet', { name: 'Android User' });
        document.getElementById('result').textContent = result;
    } catch (error) {
        console.error('Error:', error);
    }
});

if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('/sw.js')
        .then(registration => console.log('SW registered'))
        .catch(error => console.log('SW registration failed'));
}
