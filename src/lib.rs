use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if instruction_data.is_empty() {
        msg!("No instruction data provided");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Parse the instruction data
    let instruction = match OrderBookInstruction::unpack(instruction_data) {
        Ok(instruction) => instruction,
        Err(err) => {
            msg!("Failed to unpack instruction data: {:?}", err);
            return Err(err);
        }
    };

    // Route the instruction to the appropriate handler
    match instruction {
        OrderBookInstruction::PlaceOrder(order) => process_place_order(accounts, order),
        OrderBookInstruction::GetBestBuyOrder => process_get_best_buy_order(accounts),
        OrderBookInstruction::GetBestSellOrder => process_get_best_sell_order(accounts),
    }
}

// Define the possible instructions for the order book
#[derive(Debug, PartialEq)]
enum OrderBookInstruction {
    PlaceOrder(Order),
    GetBestBuyOrder,
    GetBestSellOrder,
}

impl OrderBookInstruction {
    // Pack the instruction data into a byte array
    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            OrderBookInstruction::PlaceOrder(order) => {
                buf.push(0);
                buf.extend_from_slice(&order.pack());
            }
            OrderBookInstruction::GetBestBuyOrder => {
                buf.push(1);
            }
            OrderBookInstruction::GetBestSellOrder => {
                buf.push(2);
            }
        }
        buf
    }

    // Unpack the instruction data from a byte array
    fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let tag = data.get(0).ok_or(ProgramError::InvalidInstructionData)?;
        match tag {
            0 => {
                let order = Order::unpack(&data[1..]).map_err(|err| {
                    msg!("Failed to unpack PlaceOrder instruction data: {:?}", err);
                    ProgramError::InvalidInstructionData
                })?;
                Ok(OrderBookInstruction::PlaceOrder(order))
            }
            1 => Ok(OrderBookInstruction::GetBestBuyOrder),
            2 => Ok(OrderBookInstruction::GetBestSellOrder),
            _ => {
                msg!("Invalid instruction tag");
                Err(ProgramError::InvalidInstructionData)
            }
        }
    }
}

// Define the fields of an order
#[derive(Clone, Copy, Debug, PartialEq)]
struct Order {
    trader: Pubkey,
    amount: u64,
    price: u64,
    order_type: OrderType,
}

impl Order {
    // Pack the order data into a byte array
    fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&self.trader.to_bytes());
        buf.extend_from_slice(&self.amount.to_le_bytes());
        buf.extend_from_slice(&self.price.to_le_bytes());
        buf.push(self.order_type as u8);
        buf
    }

    fn unpack(data: &[u8]) -> Result<Self, ProgramError> {
        let trader = Pubkey::new_from_array(*array_ref![data, 0, 32]);
        let amount = u64::from_le_bytes(*array_ref![data, 32, 8]);
        let price = u64::from_le_bytes(*array_ref![data, 40, 8]);
        let order_type = match data.get(48) {
            Some(0) => OrderType::Buy,
            Some(1) => OrderType::Sell,
            _ => {
                msg!("Invalid order type");
                return Err(ProgramError::InvalidAccountData);
            }
        };
        Ok(Order {
            trader,
            amount,
            price,
            order_type,
        })
    }
}

// Define the two types of orders (buy and sell)
#[derive(Clone, Copy, Debug, PartialEq)]
enum OrderType {
    Buy,
    Sell,
}

// Define the account data for the order book
#[derive(Default)]
struct OrderBook {
    buy_orders: Vec<Order>,
    sell_orders: Vec<Order>,
}

impl OrderBook {
    // Add an order to the order book
    fn add_order(&mut self, order: Order) {
        match order.order_type {
            OrderType::Buy => self.buy_orders.push(order),
            OrderType::Sell => self.sell_orders.push(order),
        }
    }

    // Get the best buy order (highest price)
    fn get_best_buy_order(&self) -> Option<&Order> {
        self.buy_orders.iter().max_by_key(|order| order.price)
    }

    // Get the best sell order (lowest price)
    fn get_best_sell_order(&self) -> Option<&Order> {
        self.sell_orders.iter().min_by_key(|order| order.price)
    }
}

// Process the PlaceOrder instruction
fn process_place_order<'a>(
    accounts: &'a [AccountInfo<'a>],
    order: Order,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let order_book_info = next_account_info(account_info_iter)?;
    let order_book = &mut OrderBook::from_account_info(order_book_info)?;

    // Add the order to the order book
    order_book.add_order(order);

    Ok(())
}

// Process the GetBestBuyOrder instruction
fn process_get_best_buy_order<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let order_book_info = next_account_info(account_info_iter)?;
    let order_book = &OrderBook::from_account_info(order_book_info)?;

    // Get the best buy order from the order book
    let best_buy_order = order_book
        .get_best_buy_order()
        .ok_or(ProgramError::InvalidAccountData)?;

    msg!("Best buy order: {:?}", best_buy_order);

    Ok(())
}

// Process the GetBestSellOrder instruction
fn process_get_best_sell<'a>(accounts: &'a [AccountInfo<'a>]) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let order_book_info = next_account_info(account_info_iter)?;
    let order_book = &OrderBook::from_account_info(order_book_info)?;
    // Get the best sell order from the order book
    let best_sell_order = order_book
    .get_best_sell_order()
    .ok_or(ProgramError::InvalidAccountData)?;

    msg!("Best sell order: {:?}", best_sell_order);

    Ok(())
}

// Define the instruction processor function
pub fn process_instruction<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = OrderBookInstruction::unpack(instruction_data)?;

    match instruction {
        OrderBookInstruction::PlaceOrder(order) => {
            msg!("Instruction: PlaceOrder");
            process_place_order(accounts, order)
        }
        OrderBookInstruction::GetBestBuyOrder => {
            msg!("Instruction: GetBestBuyOrder");
            process_get_best_buy_order(accounts)
        }
        OrderBookInstruction::GetBestSellOrder => {
            msg!("Instruction: GetBestSellOrder");
            process_get_best_sell_order(accounts)
        }
    }
}

// Declare the program ID
solana_program::declare_id!("orderbook111111111111111111111111111111111");

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        sysvar::rent,
        system_instruction,
    };

    #[test]
    fn test_order_book() {
        let program_id = solana_program::pubkey::new_rand();

        // Create the order book account
        let order_book_pubkey = solana_program::pubkey::new_rand();
        let rent = rent::Rent::default();
        let mut order_book_account = Account::new(
            rent.minimum_balance(OrderBook::LEN),
            OrderBook::LEN,
            &program_id,
        );
        OrderBook::pack_into_account(&OrderBook::default(), &mut order_book_account.data);

        let mut accounts = vec![
            AccountInfo::new(
                &solana_program::pubkey::new_rand(),
                false,
                true,
                &mut Account::default(),
            ),
            AccountInfo::new(&order_book_pubkey, false, true, &mut order_book_account),
            AccountInfo::new_readonly(&solana_program::sysvar::rent::id(), false),
        ];

        // Place a buy order
        let trader_pubkey = solana_program::pubkey::new_rand();
        let order = Order {
            trader: trader_pubkey,
            amount: 100,
            price: 500,
            order_type: OrderType::Buy,
        };
        let mut order_account = Account::new(0, Order::LEN, &program_id);
        Order::pack_into_account(&order, &mut order_account.data);
        let order_account_pubkey = solana_program::pubkey::new_rand();
        accounts.push(AccountInfo::new(
            &order_account_pubkey,
            false,
            true,
            &mut order_account,
        ));
        let place_order_ix = Instruction::new_with_bincode(
            program_id,
            &OrderBookInstruction::PlaceOrder(order),
            vec![
                AccountMeta::new(order_book_pubkey, false),
                AccountMeta::new_readonly(trader_pubkey, true),
                AccountMeta::new(order_account_pubkey, false),
            ],
        );
        assert!(solana_program_test::BanksClient::new()
            .process_instruction(&place_order_ix, accounts.clone())
            .is_ok());

       // Get the best buy order
       let get_best_buy_order_ix = Instruction::new_with_bincode(
        program_id,
        &OrderBookInstruction::GetBestBuyOrder,
        vec![AccountMeta::new_readonly(order_book_pubkey, false)],
    );
    let result = solana_program_test::BanksClient::new()
        .process_instruction(&get_best_buy_order_ix, accounts.clone())
        .unwrap();
    let best_buy_order = Order::unpack_from_slice(&result.data).unwrap();
    assert_eq!(
        best_buy_order,
        Order {
            trader: trader_pubkey,
            amount: 100,
            price: 500,
            order_type: OrderType::Buy,
        }
    );

    // Place a sell order
    let trader_pubkey = solana_program::pubkey::new_rand();
    let order = Order {
        trader: trader_pubkey,
        amount: 50,
        price: 600,
        order_type: OrderType::Sell,
    };
    let mut order_account = Account::new(0, Order::LEN, &program_id);
    Order::pack_into_account(&order, &mut order_account.data);
    let order_account_pubkey = solana_program::pubkey::new_rand();
    accounts.push(AccountInfo::new(
        &order_account_pubkey,
        false,
        true,
        &mut order_account,
    ));
    let place_order_ix = Instruction::new_with_bincode(
        program_id,
        &OrderBookInstruction::PlaceOrder(order),
        vec![
            AccountMeta::new(order_book_pubkey, false),
            AccountMeta::new_readonly(trader_pubkey, true),
            AccountMeta::new(order_account_pubkey, false),
        ],
    );
    assert!(solana_program_test::BanksClient::new()
        .process_instruction(&place_order_ix, accounts.clone())
        .is_ok());

    // Get the best sell order
    let get_best_sell_order_ix = Instruction::new_with_bincode(
        program_id,
        &OrderBookInstruction::GetBestSellOrder,
        vec![AccountMeta::new_readonly(order_book_pubkey, false)],
    );
    let result = solana_program_test::BanksClient::new()
        .process_instruction(&get_best_sell_order_ix, accounts.clone())
        .unwrap();
    let best_sell_order = Order::unpack_from_slice(&result.data).unwrap();
    assert_eq!(
        best_sell_order,
        Order {
            trader: trader_pubkey,
            amount: 50,
            price: 600,
            order_type: OrderType::Sell,
        }
    );
}
}