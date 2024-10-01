(module
  ;; Import a function that returns an integer
  (import "env" "get_number" (func $get_number (result i32)))

  ;; Define our function that uses the imported function and adds 5
  (func $add_five_to_imported (result i32)
    (i32.add
      (call $get_number)
      (i32.const 5)
    )
  )

  ;; Export our function
  (export "add_five_to_imported" (func $add_five_to_imported))
)