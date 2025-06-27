class Colors:
    HEADER = '\033[95m'      # Mor
    OKBLUE = '\033[94m'      # Mavi
    OKCYAN = '\033[96m'      # Camgöbeği
    OKGREEN = '\033[92m'     # Yeşil
    WARNING = '\033[93m'     # Sarı
    FAIL = '\033[91m'        # Kırmızı
    ENDC = '\033[0m'         # Reset
    BOLD = '\033[1m'
    UNDERLINE = '\033[4m'

__version__ = "0.1.0o1"
__versionschema_ = "nodecraft-rebuilding LFS-GOODLINE"

def get_colored_version():
    return (
        f"{Colors.BOLD}{Colors.OKGREEN}Program Version: "
        f"{Colors.OKCYAN}{__version__}{Colors.ENDC}\n"
        f"{Colors.HEADER}{__versionschema_}{Colors.ENDC}"
    )

def print_version():
    print(get_colored_version())

if __name__ == "__main__":
    print_version()
