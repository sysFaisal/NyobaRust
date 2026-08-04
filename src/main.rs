use std::vec;


#[derive(Debug)]
struct LinkList {
    value : i32,
    next : Option<Box<LinkList>>
}

impl LinkList {
    fn Create_List() -> LinkList {
        LinkList { value: -1, next: None }
    }

    fn Head_null (&self) -> bool{
        if self.value == -1 {
            return true;
        }
        return false;
    }

    fn delete_first(&mut self){
        if self.Head_null(){
            return;
        }
        
        if let Some(ref mut Temp) = self.next {
            self.value = Temp.value;
            self.next = Temp.next.take();
            return;
        }

        self.value = -1;
    }

    fn delete_last(&mut self){
        if self.Head_null(){
            return;
        }

        if self.next.is_none(){
            self.delete_first();
            return;
        }

        let mut current = self;

        // solusi kalau misalnya sisa 2 biji
        if current.next.as_ref().unwrap().next.is_none(){
            current.next = None;
            return;
        }

        // sisa 2 biji seterusnya kernel warp panic 
        while let Some(ref mut node) = current.next {
            if node.next.as_ref().unwrap().next.is_none() {
                node.next = None;
            return;
            }
            current = node;
        }


        /*
        let mut Current = self;
        
        while let Some(ref mut Node) = Current.next{
            if let Some(ref mut Node2) = Node.next {
                if Node2.next.is_none() {
                    Node.next = None;
                    return;
                }
            }
            Current = Node;
        }
        */
    }

    fn Push_first(&mut self, value:i32){

         if self.Head_null() {
            *self = LinkList { value, next: None };
            return;
        }
        
        let NewBox = Box::new(LinkList{
                value : self.value,
                next : self.next.take(),
        });

        self.value = value;
        self.next = Some(NewBox);
        
    }

    fn Push_Last(&mut self,value:i32){

        if self.Head_null() {
            *self = LinkList { value, next: None };
            return;
        }

        let NewBox = Box::new(LinkList{
            value,
            next : None,
        });

        let mut Current = self;
        while let Some(ref mut node) = Current.next {
            Current = node;
        }

        Current.next = Some(NewBox);
    }

    fn print_all(&self) {
        if self.Head_null(){
            return;
        }

        let mut Temp = self;
        while let Some(ref tmp) = Temp.next {
            println!("{}", Temp.value);
            Temp = tmp;
        }

        println!("{}", Temp.value);
    }
    
}


#[derive(Clone)]
struct Mahasiswa {
    nama : String,
    nim : String,
}


fn main() {

    let mut New = LinkList::Create_List();
    New.Push_first(50);
    New.Push_first(40);
    New.Push_first(30);
    New.Push_Last(60);
    New.print_all();
    println!("Hasil");
    New.delete_first();
    New.delete_last();
    New.delete_last();
    New.print_all();

    let mut M = Mahasiswa {
        nama : String::from("Dda"),
        nim : String::from("255008"),
    };
    let mut V = Vec::new();
    V.push(M.clone());
    V.push(M.clone());
    V.push(M.clone());    
}