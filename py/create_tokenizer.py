from tokenizers import Tokenizer
from tokenizers.models import BPE
from tokenizers.trainers import BpeTrainer
from tokenizers.pre_tokenizers import Whitespace

tokenizer = Tokenizer(BPE())
tokenizer.pre_tokenizer = Whitespace()
trainer = BpeTrainer(vocab_size=10_000, special_tokens=["<unk>"])
tokenizer.train(["pg2600.txt"], trainer)
tokenizer.save("tokenizer.json")
