mod blockchain;
use serde_json::Value;
use crate::blockchain::{Blockchain, Transaction};
extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use nwd::NwgUi;
use nwg::NativeUi;
use std::cell::RefCell;

#[derive(Default, NwgUi)]
pub struct MessageBank {
    
    #[nwg_control(size:(650, 600), position:(800, 300), title: "P2P Money Sender")]
    #[nwg_events( OnWindowClose: [MessageBank::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, max_row: Some(6), spacing: 3)]
    layout: nwg::GridLayout,

    #[nwg_control(text: "Add Peer ID", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0, col_span: 1)]
    #[nwg_events( OnButtonClick: [MessageBank::add_peer])]
    add_peer_btn: nwg::Button,

    #[nwg_control(text: "Specify $ Amount:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 1, col_span: 1)]
    add_money_btn: nwg::Label,

    //Box to add peer ID
    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 0, col_span: 1)]
    peer_id: nwg::TextInput,
    
    //Box to specify $ amount
    #[nwg_control]
    #[nwg_layout_item(layout: layout, col: 1, row: 1, col_span: 1)]
    sent_amount: nwg::TextInput,

    #[nwg_control(text:"Peer History:")]
    #[nwg_layout_item(layout: layout, col: 0, row: 3, col_span: 1)]
    friend: nwg::Label,

    #[nwg_control(text: "Send", focus: true)]
    #[nwg_layout_item(layout: layout, col: 0, row: 2, col_span: 2)]
    #[nwg_events( OnButtonClick: [MessageBank::send_money])]
    send_money: nwg::Button,
    
    #[nwg_control(text:"Current Balance: ")]
    #[nwg_layout_item(layout: layout, col: 2, row: 1, col_span: 1)]
    balance: nwg::Label,

    #[nwg_control(text:"10000")]
    #[nwg_layout_item(layout: layout, col: 3, row: 1, col_span: 1)]
    curr_balance: nwg::Label,

    boxes: RefCell<Vec<nwg::CheckBox>>,
    handlers: RefCell<Vec<nwg::EventHandler>>,
}

impl MessageBank {

    fn add_peer(&self) {
        let title = self.peer_id.text();

        self.peer_id.set_text("");

        let mut new_check = Default::default();
        nwg::CheckBox::builder()
            .text(&title)
            .parent(&self.window)
            .build(&mut new_check)
            .expect("Failed to build button");
   
        let mut boxes = self.boxes.borrow_mut();

        // new peer box positions are weird
        let blen = boxes.len() as u32;
        let (x, y) = (1+(blen % 3), 2+(blen / 3));
        self.layout.add_child(x, y+1, &new_check);

        boxes.push(new_check);
    }

    fn send_money(&self)
    {

        let mut total_amount = 0;
        let mut all_peers: String = "".to_owned();

        //check_state returns a checkbox, not a bool, so this checkbox is being used as a bool to compare the two
        let mut new_check = Default::default();
        nwg::CheckBox::builder()
            .check_state(nwg::CheckBoxState::Checked)
            .parent(&self.window)
            .build(&mut new_check)
            .expect("Failed to build button");

        let boxes = self.boxes.borrow_mut();
        let mut checks: Vec::<String> = Vec::<String>::new();
        for n in 0..boxes.len()
        {
            if boxes[n].check_state().eq(&new_check.check_state())
            {
                checks.push(boxes[n].text());
            }
        }

        if checks.len() == 0
        {
            nwg::simple_message("Error", "Please select at least 1 peer");
        }
        else if self.sent_amount.text().eq("0") || self.sent_amount.text().eq("")
        {
            nwg::simple_message("Error", "Please add an amount to send");
        }
        else if !self.sent_amount.text().chars().all(char::is_numeric)
        {
            nwg::simple_message("Error", "Sent amount must be a postive number");
        }
        else
        {
            for n in 0..checks.len()
            {
                //convert balance to int
                let curr_bal = self.curr_balance.text();
                let i: i32 = curr_bal.parse().unwrap_or(0);

                //convert sent $ to int
                let sent_amount = self.sent_amount.text();
                let j: i32 = sent_amount.parse().unwrap_or(0);

                total_amount += j;

                //update balance
                let result = i - j;
                self.curr_balance.set_text(&(result.to_string()));

                if checks.len() == 1
                {
                    all_peers.push_str(&checks[n]);
                }
                else if n < checks.len()-1
                {
                    all_peers.push_str(&checks[n]);
                    all_peers.push_str(", ");
                }
                else
                {
                    all_peers.push_str("and ");
                    all_peers.push_str(&checks[n]);
                }
            }

            let mut test_s: String = "Sent total of $".to_owned();
            test_s.push_str(&total_amount.to_string());
            test_s.push_str(" to ");
            test_s.push_str(&all_peers);

            nwg::simple_message("Transaction Successful", &test_s);
        }
        self.peer_id.set_text("");
        self.sent_amount.set_text("");
    }

    fn exit(&self) {
        let handlers = self.handlers.borrow();
        for handler in handlers.iter() {
            nwg::unbind_event_handler(&handler);
        }
        
        nwg::stop_thread_dispatch();
    }

}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let _ui = MessageBank::build_ui(Default::default()).expect("Failed to build UI");
    
    nwg::dispatch_thread_events();

}

