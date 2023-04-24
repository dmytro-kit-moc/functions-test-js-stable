// run from within app web/ directory, presumes the sqlite db file is named "database.sqlite" (default for template)

const DB_FILE_PATH = './database.sqlite';

const getAccessToken = async () => {
    return new Promise((resolve, reject) => {
        import('sqlite3').then((sqlite) => {
            const db = new sqlite.default.Database(DB_FILE_PATH);
            db.serialize(() => {
                db.get('SELECT accessToken FROM shopify_sessions LIMIT 1;', [], (err, { accessToken }) => {
                    return resolve(accessToken);
                });
            });
        });
    });
}

getAccessToken().then(console.log);
