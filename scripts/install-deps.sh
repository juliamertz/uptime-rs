DB_PATH="database.db"
NODE_BIN=./node_modules/.bin

if [ ! -f $NODE_BIN/tailwindcss ]; then
  echo "Installing tailwindcss..."
  npm install tailwindcss autoprefixer postcss
else
  echo "tailwindcss already installed"
fi

if [ ! -d "./node_modules/prettier-plugin-jinja-template" ]; then
  echo "Installing prettier-plugin-jinja-template..."
  npm install prettier-plugin-jinja-template
else
  echo "prettier-plugin-jinja-template already installed"
fi

for file in schemas/*.sql; do
  echo "Running $file"
  # cat $file
  sqlite3 $DB_PATH < $file
done
