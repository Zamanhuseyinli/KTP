from setuptools import setup, find_packages
from pathlib import Path

this_directory = Path(__file__).parent
long_description = ""
readme_path = this_directory / "README.md"
if readme_path.exists():
    long_description = readme_path.read_text(encoding="utf-8")

setup(
    name="aipropengine-ktp",
    version="0.1.0",
    description="Watcher + AI Analyzer integration for code and repo analysis",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Zaman Huseyinli",
    author_email="admin@azccriminal.space",
    url="https://github.com/Zamanhuseyinli/KTP",
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=[
        "tensorflow>=2.12.0",
        "transformers>=4.30.0",
        "gitpython>=3.1.31",
        "aioftp>=0.21.0",
        "paramiko>=3.1.0",
    ],
    classifiers=[
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: GNU General Public License v2 (GPLv2)",
        "Operating System :: OS Independent",
    ],
    license="GPLv2",
    entry_points={
        'console_scripts': [
            'aipropengine_ktp=AIpropengine.main:main',  
        ],
    },
    project_urls={
        "Homepage": "https://github.com/Zamanhuseyinli/KTP",
        "Issue Tracker": "https://github.com/Zamanhuseyinli/KTP/issues",
    },
)
