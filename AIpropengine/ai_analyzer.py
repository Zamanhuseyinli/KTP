import os
import asyncio
import json
import subprocess
from pathlib import Path
import mimetypes
import re

import tensorflow as tf
from transformers import AutoTokenizer
from transformers.models.distilbert.modeling_tf_distilbert import TFDistilBertForSequenceClassification

class FileClassifier:
    def classify_file(self, filepath: str) -> str:
        ext = Path(filepath).suffix.lower()
        if ext in ['.py', '.c', '.cpp', '.h', '.hpp', '.rs', '.java', '.js', '.ts', '.go', '.sh', '.asm', '.s', '.S']:
            return "source_code"
        elif ext in ['.md', '.txt', '.rst', '.docx', '.pdf']:
            return "document"
        elif ext in ['.json', '.yaml', '.yml', '.xml', '.ini']:
            return "config"
        else:
            mime, _ = mimetypes.guess_type(filepath)
            if mime and mime.startswith("text/"):
                return "text"
            return "binary"

class AIAnalyzer:
    def __init__(self):
        self.classifier = FileClassifier()
        self.tokenizer = AutoTokenizer.from_pretrained("distilbert-base-uncased")
        self.model = TFDistilBertForSequenceClassification.from_pretrained("distilbert-base-uncased")
        self.active_learning_memory = []

    async def analyze_local_dir(self, local_dir: str = None, gitroot_mode: str = "single"):
        """
        local_dir verilmezse GITROOT ortam değişkenine göre analiz yapar.
        gitroot_mode 'single' veya 'multiple' olabilir.
        """
        if local_dir is None:
            gitroot_env = os.environ.get("GITROOT")
            if not gitroot_env:
                print("[AIAnalyzer] GITROOT environment variable is not set.")
                return
            
            if gitroot_mode == "single":
                paths_to_analyze = [Path(gitroot_env)]
            elif gitroot_mode == "multiple":
                paths_to_analyze = [Path(p) for p in gitroot_env.split(",") if p]
            else:
                print("[AIAnalyzer] Unknown gitroot_mode, expected 'single' or 'multiple'.")
                return
        else:
            paths_to_analyze = [Path(local_dir)]

        for path in paths_to_analyze:
            if not path.exists() or not path.is_dir():
                print(f"[AIAnalyzer] Path does not exist or is not a directory: {path}")
                continue
            print(f"[AIAnalyzer] Starting analysis of directory: {path}")

            all_files = []
            for root, _, files in os.walk(path):
                for file in files:
                    all_files.append(os.path.join(root, file))

            tasks = [self.analyze_file(f) for f in all_files]
            await asyncio.gather(*tasks)

        await self._retrain_if_needed()

    async def analyze_file(self, filepath: str):
        ftype = self.classifier.classify_file(filepath)
        if ftype == "binary":
            print(f"[AIAnalyzer] Skipping binary file: {filepath}")
            return

        content = await self._read_file(filepath, ftype)
        if content is None:
            print(f"[AIAnalyzer] Could not read file: {filepath}")
            return

        print(f"[AIAnalyzer] Analyzing file: {filepath} (type: {ftype})")

        # Statik analiz
        if ftype == "source_code":
            if filepath.endswith(".py"):
                lint_issues = await self._run_pylint(filepath)
                if lint_issues:
                    print(f"[AIAnalyzer][Static Analysis][Python] Issues in {filepath}:\n{lint_issues}")
            elif filepath.endswith((".c", ".cpp", ".h", ".hpp")):
                lint_issues = await self._run_cppcheck(filepath)
                if lint_issues:
                    print(f"[AIAnalyzer][Static Analysis][C/C++] Issues in {filepath}:\n{lint_issues}")
            elif filepath.endswith((".asm", ".s", ".S")):
                lint_issues = await self._run_asm_lint(filepath)
                if lint_issues:
                    print(f"[AIAnalyzer][Static Analysis][ASM] Issues in {filepath}:\n{lint_issues}")

        # NLP içerik analizi
        prediction, confidence = await self._nlp_analyze(content)
        print(f"[AIAnalyzer][NLP] Prediction: {prediction}, Confidence: {confidence:.2f}")

        if confidence < 0.75:
            print(f"[AIAnalyzer][ActiveLearning] Low confidence on {filepath}, saving sample for retraining")
            self.active_learning_memory.append((content, prediction))

    async def _read_file(self, filepath: str, filetype: str):
        try:
            if filetype == "config":
                with open(filepath, "r", encoding="utf-8") as f:
                    content = f.read()
                try:
                    if filepath.endswith(('.json', '.yaml', '.yml')):
                        data = json.loads(content)
                        return json.dumps(data)
                except Exception:
                    return content
                return content
            else:
                with open(filepath, "r", encoding="utf-8") as f:
                    return f.read()
        except Exception as e:
            print(f"[AIAnalyzer] Error reading {filepath}: {e}")
            return None

    async def _run_pylint(self, filepath: str):
        try:
            proc = await asyncio.create_subprocess_exec(
                'pylint', filepath, '--output-format=json',
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            stdout, stderr = await proc.communicate()
            if stderr:
                print(f"[AIAnalyzer] pylint error: {stderr.decode()}")
            issues = stdout.decode()
            if issues.strip() == '[]':
                return None
            return issues
        except Exception as e:
            print(f"[AIAnalyzer] Error running pylint: {e}")
            return None

    async def _run_cppcheck(self, filepath: str):
        try:
            proc = await asyncio.create_subprocess_exec(
                'cppcheck', '--enable=all', '--error-exitcode=1', '--quiet', filepath,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
            )
            stdout, stderr = await proc.communicate()
            output = stdout.decode() + stderr.decode()
            if proc.returncode != 0 and output.strip():
                return output
            return None
        except Exception as e:
            print(f"[AIAnalyzer] Error running cppcheck: {e}")
            return None

    async def _run_asm_lint(self, filepath: str):
        try:
            with open(filepath, "r", encoding="utf-8", errors="ignore") as f:
                content = f.read()
            issues = []
            if re.search(r"\billegal\b", content, re.IGNORECASE):
                issues.append("Found 'illegal' opcode or directive.")
            # İstersen buraya ek kontrol kuralları ekleyebilirsin
            if issues:
                return "\n".join(issues)
            return None
        except Exception as e:
            print(f"[AIAnalyzer] Error reading ASM file {filepath}: {e}")
            return None

    async def _nlp_analyze(self, text: str):
        inputs = self.tokenizer(text, return_tensors="tf", truncation=True, padding=True, max_length=512)
        outputs = self.model(inputs)
        probs = tf.nn.softmax(outputs.logits, axis=-1).numpy()[0]
        classes = ["negative", "positive"]
        pred_idx = int(tf.argmax(outputs.logits, axis=-1).numpy()[0])
        confidence = probs[pred_idx]
        prediction = classes[pred_idx]
        return prediction, confidence

    async def _retrain_if_needed(self):
        if not self.active_learning_memory:
            print("[AIAnalyzer] No new samples for retraining.")
            return

        print("[AIAnalyzer] Retraining model on new samples...")

        texts, labels = zip(*self.active_learning_memory)

        # Basit örnek: tüm etiketler pozitif (1)
        labels = [1 for _ in texts]

        inputs = self.tokenizer(list(texts), return_tensors="tf", truncation=True, padding=True, max_length=512)
        labels_tf = tf.convert_to_tensor(labels)

        dataset = tf.data.Dataset.from_tensor_slices((inputs["input_ids"], labels_tf)).batch(8)

        loss_fn = tf.keras.losses.SparseCategoricalCrossentropy(from_logits=True)
        optimizer = tf.keras.optimizers.Adam(learning_rate=3e-5)

        self.model.compile(optimizer=optimizer, loss=loss_fn, metrics=["accuracy"])
        self.model.fit(dataset, epochs=1)

        print("[AIAnalyzer] Retraining complete.")
        self.active_learning_memory.clear()

    async def analyze_livestream(self, uri: str):
        print(f"[AIAnalyzer] Livestream analysis started for: {uri}")
        # Buraya streaming veri işleme eklenebilir
        await asyncio.sleep(0.1)
        print(f"[AIAnalyzer] Livestream analysis completed for: {uri}")
