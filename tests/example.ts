namespace ExampleProject {
	QUnit.module("Example");

	QUnit.test("A test title", function(assert : Assert){
		let example = new Example("name");
		assert.equal(example.value, "name", "Example() test");
	});
}
