import xpectate


def main():
    xpectate.watch('.', ['css', 'html', 'jinja'], "npx tailwindcss")


if __name__ == "__main__":
    main()
