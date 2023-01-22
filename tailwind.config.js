module.exports = {
    darkMode: 'class',
    content: [
        "./app/src/**/*.rs",
        "./app-web/src/**/*.rs",
        "./app-web/index.html"
    ],
    theme: {
        extend: {
            gridTemplateColumns: {
                'fit-keystone': 'repeat(auto-fit, minmax(min(25ch, 100%), 1fr))',
                'fit-mastery': 'repeat(auto-fit, minmax(min(40ch, 100%), 1fr))',
            }
        },
    },
    plugins: [],
}
