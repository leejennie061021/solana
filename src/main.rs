use std::env;

enum SolanaInstruction {
    InitAccount,
    Transfer(u64),
}

struct AccountData {
    is_initialized: bool,
    balance: u64,
}

fn process_instruction(instruction: SolanaInstruction, account: &mut AccountData) -> Result<(), String> {
    match instruction {
        SolanaInstruction::InitAccount => {
            if account.is_initialized {
                return Err("Аккаунт уже инициализирован".to_string());
            }
            account.is_initialized = true;
            println!("Успех: Аккаунт инициализирован.");
            Ok(())
        }
        SolanaInstruction::Transfer(amount) => {
            if !account.is_initialized {
                return Err("Аккаунт не инициализирован".to_string());
            }
            account.balance += amount;
            println!("Успех: Переведено {} токенов. Новый баланс: {}", amount, account.balance);
            Ok(())
        }
    }
}

fn main() {
    // Считываем аргументы командной строки
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Использование:");
        println!("  cargo run init");
        println!("  cargo run transfer <сумма>");
        return;
    }

    // Создаем тестовый "аккаунт"
    let mut my_account = AccountData {
        is_initialized: false,
        balance: 0,
    };

    let command = args[1].as_str();

    // Простейший парсинг CLI аргументов
    let instruction = match command {
        "init" => SolanaInstruction::InitAccount,
        "transfer" => {
            let amount = args.get(2)
                .expect("Укажите сумму для перевода")
                .parse::<u64>()
                .expect("Сумма должна быть числом");
            SolanaInstruction::Transfer(amount)
        }
        _ => {
            println!("Неизвестная команда!");
            return;
        }
    };

    // Выполнение инструкции и обработка результата
    match process_instruction(instruction, &mut my_account) {
        Ok(_) => println!("Транзакция выполнена успешно."),
        Err(e) => println!("Ошибка транзакции: {}", e),
    }
}